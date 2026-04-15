#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="docker/cassandra/docker-compose.yml"
MAX_WAIT_SECONDS=180
STATUS_REFRESH_SECONDS=2
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
SERVICE_IPS=()
SERVICE_STATES=()
STATUS_SOURCE=""
LAST_STATUS_REFRESH=0
DRAWN_LINES=0

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

get_container_id() {
  local service="$1"
  docker compose -f "$COMPOSE_FILE" ps -q "$service" 2>/dev/null || true
}

get_container_ip() {
  local cid="$1"
  [[ -n "$cid" ]] || return 1
  docker inspect -f '{{range $name, $net := .NetworkSettings.Networks}}{{if $net.IPAddress}}{{printf "%s\n" $net.IPAddress}}{{end}}{{end}}' "$cid" \
    | head -n1
}

init_services() {
  SERVICES=()
  SERVICE_IPS=()
  SERVICE_STATES=()

  while IFS= read -r svc; do
    [[ -n "$svc" ]] || continue
    SERVICES+=("$svc")

    local cid ip
    cid="$(get_container_id "$svc")"
    ip="$(get_container_ip "$cid" 2>/dev/null || true)"

    SERVICE_IPS+=("$ip")
    SERVICE_STATES+=("waiting")
  done < <(get_services)

  if [[ "${#SERVICES[@]}" -gt 0 ]]; then
    STATUS_SOURCE="${SERVICES[0]}"
  fi
}

refresh_service_ips() {
  local i svc cid ip
  for ((i=0; i<${#SERVICES[@]}; i++)); do
    svc="${SERVICES[$i]}"
    cid="$(get_container_id "$svc")"
    ip="$(get_container_ip "$cid" 2>/dev/null || true)"
    SERVICE_IPS[$i]="$ip"
  done
}

refresh_cluster_status() {
  local now
  now="$(date +%s)"

  if (( now - LAST_STATUS_REFRESH < STATUS_REFRESH_SECONDS )); then
    return 0
  fi
  LAST_STATUS_REFRESH="$now"

  refresh_service_ips

  local raw=""
  if [[ -n "$STATUS_SOURCE" ]]; then
    raw="$(docker exec "$STATUS_SOURCE" nodetool status 2>/dev/null || true)"
  fi

  local i ip state
  for ((i=0; i<${#SERVICES[@]}; i++)); do
    ip="${SERVICE_IPS[$i]}"

    if [[ -z "$ip" ]]; then
      SERVICE_STATES[$i]="missing"
      continue
    fi

    state="$(awk -v target_ip="$ip" '
      $1 ~ /^(UN|UJ|UM|DN|DJ|DM|MN|MJ|MM|NL|UL)$/ && $2 == target_ip {
        print $1
        found=1
        exit
      }
      END {
        if (!found) print "waiting"
      }
    ' <<< "$raw")"

    SERVICE_STATES[$i]="$state"
  done
}

count_ready_nodes() {
  local ready=0
  local state
  for state in "${SERVICE_STATES[@]}"; do
    [[ "$state" == "UN" ]] && ready=$((ready + 1))
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
  printf "\033[K${CYAN}Waiting for Cassandra cluster to be restored...${RESET}\n"
  lines=$((lines + 1))

  printf "\033[K${BOLD}Nodes up: ${CYAN}%d/%d${RESET}\n" "$ready_count" "$total_count"
  lines=$((lines + 1))

  local i svc state
  for ((i=0; i<${#SERVICES[@]}; i++)); do
    svc="${SERVICES[$i]}"
    state="${SERVICE_STATES[$i]}"

    case "$state" in
      UN)
        printf "\033[K ${GREEN}✔${RESET} %s ${DIM}(ready)${RESET}\n" "$svc"
        ;;
      UJ|UM)
        printf "\033[K ${YELLOW}%s${RESET} %s ${DIM}(joining cluster)${RESET}\n" "$spinner" "$svc"
        ;;
      DN|DJ|DM)
        printf "\033[K ${RED}✖${RESET} %s ${DIM}(down)${RESET}\n" "$svc"
        ;;
      missing)
        printf "\033[K ${YELLOW}%s${RESET} %s ${DIM}(starting)${RESET}\n" "$spinner" "$svc"
        ;;
      waiting)
        printf "\033[K ${YELLOW}%s${RESET} %s ${DIM}(starting)${RESET}\n" "$spinner" "$svc"
        ;;
      *)
        printf "\033[K ${YELLOW}%s${RESET} %s ${DIM}(%s)${RESET}\n" "$spinner" "$svc" "$state"
        ;;
    esac

    lines=$((lines + 1))
  done

  DRAWN_LINES="$lines"
}

main() {
  echo "Restoring Cassandra cluster to clean state..."
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
      printf "\n${GREEN}✔ Cluster restored successfully.${RESET}\n"
      break
    fi

    local now elapsed
    now="$(date +%s)"
    elapsed=$((now - start_ts))

    if (( elapsed >= MAX_WAIT_SECONDS )); then
      printf "\n${RED}✖ ERROR: Cluster did not recover after %ss.${RESET}\n" "$MAX_WAIT_SECONDS"
      exit 1
    fi

    sleep "$SPINNER_INTERVAL"
  done
}

main "$@"