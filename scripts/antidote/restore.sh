#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="docker/antidote/docker-compose.yml"
MAX_WAIT_SECONDS=180
STATUS_REFRESH_SECONDS=1
SPINNER_INTERVAL=0.1

SPINNER_FRAMES=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
SPINNER_INDEX=0

YELLOW='\033[1;33m'
GREEN='\033[1;32m'
RED='\033[1;31m'
CYAN='\033[1;36m'
DIM='\033[2m'
BOLD='\033[1m'
RESET='\033[0m'

SERVICES=()
SERVICE_STATES=()
DRAWN_LINES=0
LAST_STATUS_REFRESH=0

hide_cursor() { tput civis 2>/dev/null || true; }
show_cursor() { tput cnorm 2>/dev/null || true; }

cleanup() {
  show_cursor
  printf '\n'
}
trap cleanup EXIT

get_services() {
  docker compose -f "$COMPOSE_FILE" ps --services
}

init_services() {
  SERVICES=()
  SERVICE_STATES=()

  while IFS= read -r svc; do
    [[ -n "$svc" ]] || continue
    SERVICES+=("$svc")
    SERVICE_STATES+=("starting")
  done < <(get_services)
}

check_service_health() {
  local service="$1"
  local cid
  cid="$(docker compose -f "$COMPOSE_FILE" ps -q "$service" 2>/dev/null || true)"

  if [[ -z "$cid" ]]; then
    echo "starting"
    return 0
  fi

  local running
  running="$(docker inspect -f '{{.State.Running}}' "$cid" 2>/dev/null || true)"
  if [[ "$running" != "true" ]]; then
    echo "starting"
    return 0
  fi

  # This image reports startup with lines like: "started_at: antidote@antidote1"
  # Avoid grep -q with pipefail (docker logs may get SIGPIPE and look like failure).
  if docker logs "$cid" 2>&1 | grep -F "started_at: antidote@${service}" >/dev/null; then
    echo "ready"
  else
    echo "starting"
  fi
}

refresh_cluster_status() {
  local now
  now="$(date +%s)"

  if (( now - LAST_STATUS_REFRESH < STATUS_REFRESH_SECONDS )); then
    return 0
  fi
  LAST_STATUS_REFRESH="$now"

  local i svc
  for ((i=0; i<${#SERVICES[@]}; i++)); do
    svc="${SERVICES[$i]}"
    SERVICE_STATES[$i]="$(check_service_health "$svc")"
  done
}

count_ready_nodes() {
  local ready=0
  local state
  for state in "${SERVICE_STATES[@]}"; do
    [[ "$state" == "ready" ]] && ready=$((ready + 1))
  done
  echo "$ready"
}

draw_ui() {
  local spinner="$1"
  local ready_count="$2"
  local total_count="$3"

  if (( DRAWN_LINES > 0 )); then
    printf '\033[%dA' "$DRAWN_LINES"
  fi

  local lines=0
  printf "\033[K${CYAN}Waiting for Antidote cluster to be restored...${RESET}\n"
  lines=$((lines + 1))

  printf "\033[K${BOLD}Nodes up: ${CYAN}%d/%d${RESET}\n" "$ready_count" "$total_count"
  lines=$((lines + 1))

  local i svc state
  for ((i=0; i<${#SERVICES[@]}; i++)); do
    svc="${SERVICES[$i]}"
    state="${SERVICE_STATES[$i]}"

    case "$state" in
      ready)
        printf "\033[K ${GREEN}✔${RESET} %s ${DIM}(ready)${RESET}\n" "$svc"
        ;;
      missing)
        printf "\033[K ${RED}✖${RESET} %s ${DIM}(container not found)${RESET}\n" "$svc"
        ;;
      *)
        printf "\033[K ${YELLOW}%s${RESET} %s ${DIM}(starting)${RESET}\n" "$spinner" "$svc"
        ;;
    esac

    lines=$((lines + 1))
  done

  DRAWN_LINES="$lines"
}

main() {
  echo "Restoring Antidote cluster to clean state..."
  docker compose -f "$COMPOSE_FILE" down -v

  sleep 3

  docker compose -f "$COMPOSE_FILE" up -d

  init_services

  if [[ "${#SERVICES[@]}" -eq 0 ]]; then
    echo "No services found in compose file."
    exit 1
  fi

  hide_cursor

  local start_ts
  start_ts="$(date +%s)"

  while true; do
    refresh_cluster_status

    local ready_count
    ready_count="$(count_ready_nodes)"

    local spinner="${SPINNER_FRAMES[$SPINNER_INDEX]}"
    SPINNER_INDEX=$(( (SPINNER_INDEX + 1) % ${#SPINNER_FRAMES[@]} ))

    draw_ui "$spinner" "$ready_count" "${#SERVICES[@]}"

    if [[ "$ready_count" -eq "${#SERVICES[@]}" ]]; then
      printf "\n${GREEN}✔ Antidote cluster restored successfully.${RESET}\n"
      break
    fi

    local now elapsed
    now="$(date +%s)"
    elapsed=$((now - start_ts))

    if (( elapsed >= MAX_WAIT_SECONDS )); then
      printf "\n${RED}✖ ERROR: Antidote cluster did not recover after %ss.${RESET}\n" "$MAX_WAIT_SECONDS"
      exit 1
    fi

    sleep "$SPINNER_INTERVAL"
  done
}

main "$@"