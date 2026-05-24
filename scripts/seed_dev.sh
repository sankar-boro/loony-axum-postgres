#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# seed_dev.sh — populate the dev environment with sample data.
# Creates 2 users, 2 blogs (with nodes), 2 books (with chapters & sections),
# and cross-follow between the two users.
#
# Requires: curl, jq
# Usage:    ./scripts/seed_dev.sh
#
# Env vars:
#   AUTH_URL  (default https://localhost:8000)
#   API_URL   (default http://localhost:8003)
# ---------------------------------------------------------------------------

AUTH_URL="${AUTH_URL:-https://localhost:8000}"
API_URL="${API_URL:-http://localhost:8003}"

COOKIE_JAR1="$(mktemp /tmp/seed_u1.XXXXXX)"
COOKIE_JAR2="$(mktemp /tmp/seed_u2.XXXXXX)"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

cleanup() { rm -f "$COOKIE_JAR1" "$COOKIE_JAR2"; }
trap cleanup EXIT

ok()   { echo -e "  ${GREEN}✔${NC}  $*"; }
err()  { echo -e "  ${RED}✖${NC}  $*"; }
info() { echo -e "\n${CYAN}${BOLD}──${NC} $*"; }

# ---------------------------------------------------------------------------
# HTTP helpers
# ---------------------------------------------------------------------------
auth_req() {
    local jar="$1" method="$2" path="$3"; shift 3
    curl -k -s -X "$method" \
         -H "Content-Type: application/json" \
         -c "$jar" -b "$jar" \
         "$@" "${AUTH_URL}${path}"
}

api_req() {
    local jar="$1" method="$2" path="$3"; shift 3
    curl -s -X "$method" \
         -H "Content-Type: application/json" \
         -c "$jar" -b "$jar" \
         "$@" "${API_URL}${path}"
}

check() {
    local label="$1" resp="$2" ok_code="${3:-200}"
    local code; code=$(echo "$resp" | python3 -c "import sys; lines=sys.stdin.read().splitlines(); print(lines[-1])" 2>/dev/null || echo "000")
    if [ "$code" = "$ok_code" ]; then
        ok "$label"
    else
        err "$label (HTTP $code)"
        echo "     $(echo "$resp" | head -c 200)"
    fi
    # return body
    echo "$resp" | head -n -1
}

# ---------------------------------------------------------------------------
# 1. Create users
# ---------------------------------------------------------------------------
info "1. Create users"

USERS=(
    '{"username":"alice@loony.dev","password":"AlicePassword1!","fname":"Alice","lname":"Smith"}'
    '{"username":"bob@loony.dev","password":"BobPassword1!","fname":"Bob","lname":"Jones"}'
)
JARS=("$COOKIE_JAR1" "$COOKIE_JAR2")
NAMES=("alice" "bob")

declare -a USER_IDS

for i in 0 1; do
    RESP=$(auth_req "${JARS[$i]}" POST /signup -d "${USERS[$i]}" -w "\n%{http_code}")
    CODE=$(echo "$RESP" | tail -1)
    if [ "$CODE" = "200" ]; then
        ok "signup: ${NAMES[$i]}"
    elif [ "$CODE" = "400" ] || [ "$CODE" = "409" ]; then
        ok "signup: ${NAMES[$i]} (already exists)"
    else
        err "signup: ${NAMES[$i]} (HTTP $CODE)"
    fi
done

# ---------------------------------------------------------------------------
# 2. Login
# ---------------------------------------------------------------------------
info "2. Login"

PASSWORDS=("AlicePassword1!" "BobPassword1!")
EMAILS=("alice@loony.dev" "bob@loony.dev")

for i in 0 1; do
    RESP=$(auth_req "${JARS[$i]}" POST /login \
        -d "{\"username\":\"${EMAILS[$i]}\",\"password\":\"${PASSWORDS[$i]}\"}" \
        -w "\n%{http_code}")
    CODE=$(echo "$RESP" | tail -1)
    if [ "$CODE" = "200" ]; then
        ok "login: ${NAMES[$i]}"
    else
        err "login: ${NAMES[$i]} (HTTP $CODE) — cannot continue"
        exit 1
    fi
    # get user id
    INFO=$(auth_req "${JARS[$i]}" GET /user/userInfo)
    USER_IDS[$i]=$(echo "$INFO" | jq -r '.uid // .id // empty' 2>/dev/null)
    ok "${NAMES[$i]} user_id=${USER_IDS[$i]:-unknown}"
done

ALICE_ID="${USER_IDS[0]}"
BOB_ID="${USER_IDS[1]}"

# ---------------------------------------------------------------------------
# 3. Alice creates 2 blogs
# ---------------------------------------------------------------------------
info "3. Alice creates blogs"

BLOG1=$(api_req "$COOKIE_JAR1" POST /blog/create \
    -d '{"title":"Getting Started with Rust","content":"<basic> Rust is a systems programming language focused on safety, speed, and concurrency.","images":[],"tags":["rust","programming"]}' \
    -w "\n%{http_code}")
BODY=$(check "blog 1: Getting Started with Rust" "$BLOG1")
BLOG1_ID=$(echo "$BODY" | jq -r '.doc_id // empty' 2>/dev/null)

BLOG2=$(api_req "$COOKIE_JAR1" POST /blog/create \
    -d '{"title":"Building REST APIs with Axum","content":"<basic> Axum is a web application framework that focuses on ergonomics and modularity.","images":[],"tags":["axum","rust","api"]}' \
    -w "\n%{http_code}")
BODY=$(check "blog 2: Building REST APIs with Axum" "$BLOG2")
BLOG2_ID=$(echo "$BODY" | jq -r '.doc_id // empty' 2>/dev/null)

echo "  blog1_id=$BLOG1_ID  blog2_id=$BLOG2_ID"

# ---------------------------------------------------------------------------
# 4. Append nodes to blog 1
# ---------------------------------------------------------------------------
info "4. Append nodes to blog 1"

if [ -n "$BLOG1_ID" ]; then
    # Get the main node uid first
    NODES=$(api_req "$COOKIE_JAR1" GET "/blog/get/nodes?doc_id=${BLOG1_ID}")
    MAIN_UID=$(echo "$NODES" | jq -r '.main_node.uid // empty' 2>/dev/null)

    for section in \
        "Installation and Setup|<basic> Install Rust via rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" \
        "Your First Program|<basic> Create a new project with cargo new hello_world and run it with cargo run." \
        "Ownership and Borrowing|<basic> Rust's ownership system guarantees memory safety without a garbage collector."; do
        TITLE="${section%%|*}"
        CONTENT="${section##*|}"
        if [ -n "$MAIN_UID" ]; then
            RESP=$(api_req "$COOKIE_JAR1" POST /blog/append/node \
                -d "{\"doc_id\":${BLOG1_ID},\"parent_id\":${MAIN_UID},\"title\":\"${TITLE}\",\"content\":\"${CONTENT}\",\"images\":[],\"tags\":null}" \
                -w "\n%{http_code}")
            BODY=$(check "  node: $TITLE" "$RESP")
            MAIN_UID=$(echo "$BODY" | jq -r '.new_node.uid // empty' 2>/dev/null)
        fi
    done
else
    err "blog1_id missing — skipping nodes"
fi

# ---------------------------------------------------------------------------
# 5. Bob creates 2 books
# ---------------------------------------------------------------------------
info "5. Bob creates books"

BOOK1=$(api_req "$COOKIE_JAR2" POST /book/create \
    -d '{"title":"The Complete Guide to PostgreSQL","content":"<basic> A comprehensive guide to PostgreSQL for developers and DBAs.","images":[],"tags":["postgresql","database"]}' \
    -w "\n%{http_code}")
BODY=$(check "book 1: The Complete Guide to PostgreSQL" "$BOOK1")
BOOK1_ID=$(echo "$BODY" | jq -r '.doc_id // empty' 2>/dev/null)

BOOK2=$(api_req "$COOKIE_JAR2" POST /book/create \
    -d '{"title":"Async Programming in Rust","content":"<basic> An in-depth look at async/await in Rust using Tokio.","images":[],"tags":["rust","async","tokio"]}' \
    -w "\n%{http_code}")
BODY=$(check "book 2: Async Programming in Rust" "$BOOK2")
BOOK2_ID=$(echo "$BODY" | jq -r '.doc_id // empty' 2>/dev/null)

echo "  book1_id=$BOOK1_ID  book2_id=$BOOK2_ID"

# ---------------------------------------------------------------------------
# 6. Add chapters and sections to book 1
# ---------------------------------------------------------------------------
info "6. Add chapters and sections to book 1"

if [ -n "$BOOK1_ID" ]; then
    NAV=$(api_req "$COOKIE_JAR2" GET "/book/get/nav?doc_id=${BOOK1_ID}")
    FRONT_UID=$(echo "$NAV" | jq -r '.main_node.uid // empty' 2>/dev/null)
    echo "  front_page_uid=$FRONT_UID"

    CHAPTERS=(
        "Introduction to PostgreSQL"
        "Data Types and Schema Design"
        "Querying with SQL"
        "Indexes and Performance"
    )

    PREV_UID="$FRONT_UID"

    for chapter in "${CHAPTERS[@]}"; do
        if [ -n "$PREV_UID" ]; then
            RESP=$(api_req "$COOKIE_JAR2" POST /book/append/node \
                -d "{\"doc_id\":${BOOK1_ID},\"parent_id\":${PREV_UID},\"page_id\":${PREV_UID},\"title\":\"${chapter}\",\"content\":\"<basic> ${chapter} overview.\",\"images\":[],\"identity\":101,\"parent_identity\":100,\"tags\":null}" \
                -w "\n%{http_code}")
            BODY=$(check "  chapter: $chapter" "$RESP")
            CHAPTER_UID=$(echo "$BODY" | jq -r '.new_node.uid // empty' 2>/dev/null)

            # Add 2 sections to each chapter
            SECTION_PREV="$CHAPTER_UID"
            for j in 1 2; do
                if [ -n "$SECTION_PREV" ] && [ -n "$CHAPTER_UID" ]; then
                    RESP=$(api_req "$COOKIE_JAR2" POST /book/append/node \
                        -d "{\"doc_id\":${BOOK1_ID},\"parent_id\":${SECTION_PREV},\"page_id\":${CHAPTER_UID},\"title\":\"${chapter} — Part ${j}\",\"content\":\"<basic> Detailed content for part ${j}.\",\"images\":[],\"identity\":102,\"parent_identity\":101,\"tags\":null}" \
                        -w "\n%{http_code}")
                    BODY=$(check "    section: $chapter — Part $j" "$RESP")
                    SECTION_PREV=$(echo "$BODY" | jq -r '.new_node.uid // empty' 2>/dev/null)
                fi
            done

            PREV_UID="$CHAPTER_UID"
        fi
    done
else
    err "book1_id missing — skipping chapters"
fi

# ---------------------------------------------------------------------------
# 7. Follow each other
# ---------------------------------------------------------------------------
info "7. Follow each other"

if [ -n "$BOB_ID" ]; then
    RESP=$(api_req "$COOKIE_JAR1" POST "/user/${BOB_ID}/subscribe" -w "\n%{http_code}")
    check "alice follows bob" "$RESP" > /dev/null
fi

if [ -n "$ALICE_ID" ]; then
    RESP=$(api_req "$COOKIE_JAR2" POST "/user/${ALICE_ID}/subscribe" -w "\n%{http_code}")
    check "bob follows alice" "$RESP" > /dev/null
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}${BOLD}Seed complete.${NC}"
echo ""
echo -e "  ${BOLD}Users${NC}"
echo    "    alice@loony.dev   id=${ALICE_ID:-?}  pw=AlicePassword1!"
echo    "    bob@loony.dev     id=${BOB_ID:-?}   pw=BobPassword1!"
echo ""
echo -e "  ${BOLD}Blogs (alice)${NC}"
echo    "    blog1_id=${BLOG1_ID:-?}  — Getting Started with Rust"
echo    "    blog2_id=${BLOG2_ID:-?}  — Building REST APIs with Axum"
echo ""
echo -e "  ${BOLD}Books (bob)${NC}"
echo    "    book1_id=${BOOK1_ID:-?}  — The Complete Guide to PostgreSQL"
echo    "    book2_id=${BOOK2_ID:-?}  — Async Programming in Rust"
echo ""
