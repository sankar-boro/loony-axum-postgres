#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# test_api.sh — end-to-end tests for loony-book-backend.
# Covers: auth login, blog CRUD, book CRUD, user follow/unfollow.
#
# Requires: curl, jq
# Usage:    ./scripts/test_api.sh
#
# Env vars (all optional — defaults shown):
#   AUTH_URL   base URL of loony-auth   (default https://localhost:8000)
#   API_URL    base URL of loony-api    (default http://localhost:8003)
#   TEST_EMAIL login email              (default test@example.com)
#   TEST_PASS  login password           (default TestPassword123!)
#   TEST_USER2_EMAIL  second user for follow tests (default test2@example.com)
#   TEST_USER2_PASS                                (default TestPassword123!)
# ---------------------------------------------------------------------------

AUTH_URL="${AUTH_URL:-https://localhost:8000}"
API_URL="${API_URL:-http://localhost:8003}"
TEST_EMAIL="${TEST_EMAIL:-test@example.com}"
TEST_PASS="${TEST_PASS:-TestPassword123!}"
TEST_USER2_EMAIL="${TEST_USER2_EMAIL:-test2@example.com}"
TEST_USER2_PASS="${TEST_USER2_PASS:-TestPassword123!}"

COOKIE_JAR="$(mktemp /tmp/api_cookies_1.XXXXXX)"
COOKIE_JAR2="$(mktemp /tmp/api_cookies_2.XXXXXX)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

BLOG_ID=""
BLOG_NODE_ID=""
BOOK_ID=""
BOOK_CHAPTER_ID=""
BOOK_SECTION_ID=""
USER1_ID=""
USER2_ID=""

cleanup() { rm -f "$COOKIE_JAR" "$COOKIE_JAR2"; }
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
auth_req() {
    local method="$1"; shift
    local path="$1";   shift
    curl -k -s -w "\n%{http_code}" \
         -X "$method" \
         -H "Content-Type: application/json" \
         -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
         "$@" \
         "${AUTH_URL}${path}"
}

api_req() {
    local method="$1"; shift
    local path="$1";   shift
    curl -s -w "\n%{http_code}" \
         -X "$method" \
         -H "Content-Type: application/json" \
         -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
         "$@" \
         "${API_URL}${path}"
}

# Second user (for follow tests)
api_req2() {
    local method="$1"; shift
    local path="$1";   shift
    curl -s -w "\n%{http_code}" \
         -X "$method" \
         -H "Content-Type: application/json" \
         -c "$COOKIE_JAR2" -b "$COOKIE_JAR2" \
         "$@" \
         "${API_URL}${path}"
}

res_body()   { echo "$1" | head -n -1; }
res_status() { echo "$1" | tail -n 1; }

pass() { echo -e "${GREEN}PASS${NC}  $*"; PASS=$((PASS + 1)); }
fail() { echo -e "${RED}FAIL${NC}  $*"; FAIL=$((FAIL + 1)); }
skip() { echo -e "${YELLOW}SKIP${NC}  $*"; SKIP=$((SKIP + 1)); }
info() { echo -e "${CYAN}${BOLD}───${NC} $*"; }

assert_status() {
    local name="$1" expected="$2" resp="$3"
    local body status
    body=$(res_body "$resp")
    status=$(res_status "$resp")
    if [ "$status" = "$expected" ]; then
        pass "[$name] HTTP $status"
    else
        fail "[$name] expected HTTP $expected, got HTTP $status"
        echo "       body: $(echo "$body" | head -c 300)"
    fi
    echo "$body"
}

extract() {
    # extract <json_body> <jq_filter>
    echo "$1" | jq -r "$2" 2>/dev/null
}

# ---------------------------------------------------------------------------
# 0. Signup — create user1 and user2 if they don't exist yet
# ---------------------------------------------------------------------------
echo ""
info "0. Signup (idempotent)"

RESP=$(auth_req POST /signup -d "{\"username\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASS\",\"fname\":\"Test\",\"lname\":\"One\"}")
STATUS=$(res_status "$RESP")
if [ "$STATUS" = "200" ]; then
    pass "[signup: user1 created]"
elif [ "$STATUS" = "400" ] || [ "$STATUS" = "409" ]; then
    pass "[signup: user1 already exists (skipping)]"
else
    fail "[signup: user1 unexpected status $STATUS]"
fi

RESP=$(auth_req POST /signup -d "{\"username\":\"$TEST_USER2_EMAIL\",\"password\":\"$TEST_USER2_PASS\",\"fname\":\"Test\",\"lname\":\"Two\"}")
STATUS=$(res_status "$RESP")
if [ "$STATUS" = "200" ]; then
    pass "[signup: user2 created]"
elif [ "$STATUS" = "400" ] || [ "$STATUS" = "409" ]; then
    pass "[signup: user2 already exists (skipping)]"
else
    fail "[signup: user2 unexpected status $STATUS]"
fi

# ---------------------------------------------------------------------------
# 1. Login
# ---------------------------------------------------------------------------
echo ""
info "1. Login"

RESP=$(auth_req POST /login -d "{\"username\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASS\"}")
BODY=$(res_body "$RESP"); STATUS=$(res_status "$RESP")
if [ "$STATUS" = "200" ]; then
    pass "[login: user1]"
    USER1_ID=$(extract "$BODY" '.uid // .id // .user_id // empty')
else
    fail "[login: user1] HTTP $STATUS — body: $(echo "$BODY" | head -c 200)"
    echo -e "${RED}Cannot continue without auth — aborting.${NC}"
    exit 1
fi

# Get user1 id from /user/userInfo
RESP=$(auth_req GET /user/userInfo)
BODY=$(res_body "$RESP"); STATUS=$(res_status "$RESP")
assert_status "userInfo: user1" 200 "$RESP" > /dev/null
if [ "$STATUS" = "200" ]; then
    USER1_ID=$(extract "$BODY" '.uid // .id // empty')
fi

# Login user2 into second cookie jar (share the AUTH jar temporarily)
SAVED="$COOKIE_JAR"
COOKIE_JAR="$COOKIE_JAR2"
RESP=$(auth_req POST /login -d "{\"username\":\"$TEST_USER2_EMAIL\",\"password\":\"$TEST_USER2_PASS\"}")
STATUS=$(res_status "$RESP")
if [ "$STATUS" = "200" ]; then
    pass "[login: user2]"
    RESP2=$(auth_req GET /user/userInfo)
    USER2_ID=$(extract "$(res_body "$RESP2")" '.uid // .id // empty')
else
    fail "[login: user2] HTTP $STATUS"
fi
COOKIE_JAR="$SAVED"

echo "       user1_id=${USER1_ID:-unknown}  user2_id=${USER2_ID:-unknown}"

# ---------------------------------------------------------------------------
# 2. Public endpoints — no auth needed
# ---------------------------------------------------------------------------
echo ""
info "2. Public read endpoints"

RESP=$(api_req GET /blog/get/home_blogs)
assert_status "GET /blog/get/home_blogs" 200 "$RESP" > /dev/null

RESP=$(api_req GET /blog/get/1/by_page)
assert_status "GET /blog/get/1/by_page" 200 "$RESP" > /dev/null

RESP=$(api_req GET /book/get/home_books)
assert_status "GET /book/get/home_books" 200 "$RESP" > /dev/null

RESP=$(api_req GET /book/get/1/by_page)
assert_status "GET /book/get/1/by_page" 200 "$RESP" > /dev/null

if [ -n "$USER1_ID" ]; then
    RESP=$(api_req GET "/blog/get/${USER1_ID}/user_blogs")
    assert_status "GET /blog/get/:uid/user_blogs" 200 "$RESP" > /dev/null

    RESP=$(api_req GET "/book/get/${USER1_ID}/user_books")
    assert_status "GET /book/get/:uid/user_books" 200 "$RESP" > /dev/null

    RESP=$(api_req GET "/blog/get/${USER1_ID}/get_users_blog")
    assert_status "GET /blog/get/:user_id/get_users_blog" 200 "$RESP" > /dev/null

    RESP=$(api_req GET "/book/get/${USER1_ID}/get_users_book")
    assert_status "GET /book/get/:user_id/get_users_book" 200 "$RESP" > /dev/null
fi

# ---------------------------------------------------------------------------
# 3. Blog — create
# ---------------------------------------------------------------------------
echo ""
info "3. Blog — create"

RESP=$(api_req POST /blog/create -d '{
    "title": "My First Blog Post",
    "content": "<basic> This is the main content of my blog.",
    "images": [],
    "tags": ["test", "loony"]
}')
BODY=$(res_body "$RESP"); STATUS=$(res_status "$RESP")
assert_status "POST /blog/create" 200 "$RESP" > /dev/null
BLOG_ID=$(extract "$BODY" '.doc_id // .uid // empty')
BLOG_MAIN_NODE_UID=$(extract "$BODY" '.uid // .doc_id // empty')
echo "       blog_id=${BLOG_ID:-unknown}"

# ---------------------------------------------------------------------------
# 4. Blog — read nodes
# ---------------------------------------------------------------------------
echo ""
info "4. Blog — read"

if [ -n "$BLOG_ID" ]; then
    RESP=$(api_req GET "/blog/get/nodes?doc_id=${BLOG_ID}")
    BODY=$(res_body "$RESP")
    assert_status "GET /blog/get/nodes?doc_id=$BLOG_ID" 200 "$RESP" > /dev/null
    BLOG_MAIN_NODE_UID=$(extract "$BODY" '.main_node.uid // empty')
else
    skip "[blog read] no blog_id"
fi

# ---------------------------------------------------------------------------
# 5. Blog — edit main node
# ---------------------------------------------------------------------------
echo ""
info "5. Blog — edit main"

if [ -n "$BLOG_ID" ] && [ -n "$BLOG_MAIN_NODE_UID" ]; then
    RESP=$(api_req POST /blog/edit/main -d "{
        \"uid\": ${BLOG_MAIN_NODE_UID},
        \"doc_id\": ${BLOG_ID},
        \"title\": \"My Updated Blog Post\",
        \"content\": \"<basic> Updated content.\",
        \"images\": []
    }")
    assert_status "POST /blog/edit/main" 200 "$RESP" > /dev/null
else
    skip "[blog edit main] missing blog_id or main_node uid"
fi

# ---------------------------------------------------------------------------
# 6. Blog — append node
# ---------------------------------------------------------------------------
echo ""
info "6. Blog — append node"

if [ -n "$BLOG_ID" ] && [ -n "$BLOG_MAIN_NODE_UID" ]; then
    RESP=$(api_req POST /blog/append/node -d "{
        \"doc_id\": ${BLOG_ID},
        \"parent_id\": ${BLOG_MAIN_NODE_UID},
        \"title\": \"Section 1\",
        \"content\": \"<basic> First section content.\",
        \"images\": [],
        \"tags\": null
    }")
    BODY=$(res_body "$RESP")
    assert_status "POST /blog/append/node" 200 "$RESP" > /dev/null
    BLOG_NODE_ID=$(extract "$BODY" '.new_node.uid // empty')
    echo "       blog_node_id=${BLOG_NODE_ID:-unknown}"
else
    skip "[blog append node] missing blog_id or parent uid"
fi

# ---------------------------------------------------------------------------
# 7. Blog — edit node
# ---------------------------------------------------------------------------
echo ""
info "7. Blog — edit node"

if [ -n "$BLOG_NODE_ID" ] && [ -n "$BLOG_ID" ]; then
    RESP=$(api_req POST /blog/edit/node -d "{
        \"uid\": ${BLOG_NODE_ID},
        \"doc_id\": ${BLOG_ID},
        \"title\": \"Section 1 (edited)\",
        \"content\": \"<basic> Updated section content.\",
        \"images\": []
    }")
    assert_status "POST /blog/edit/node" 200 "$RESP" > /dev/null
else
    skip "[blog edit node] missing blog_node_id"
fi

# ---------------------------------------------------------------------------
# 8. Blog — delete node
# ---------------------------------------------------------------------------
echo ""
info "8. Blog — delete node"

if [ -n "$BLOG_NODE_ID" ]; then
    RESP=$(api_req POST /blog/delete/node -d "{
        \"delete_node\": { \"uid\": ${BLOG_NODE_ID} },
        \"update_node\": null
    }")
    assert_status "POST /blog/delete/node" 200 "$RESP" > /dev/null
else
    skip "[blog delete node] missing blog_node_id"
fi

# ---------------------------------------------------------------------------
# 9. Book — create
# ---------------------------------------------------------------------------
echo ""
info "9. Book — create"

RESP=$(api_req POST /book/create -d '{
    "title": "My First Book",
    "content": "<basic> Introduction to my book.",
    "images": [],
    "tags": ["test", "book"]
}')
BODY=$(res_body "$RESP"); STATUS=$(res_status "$RESP")
assert_status "POST /book/create" 200 "$RESP" > /dev/null
BOOK_ID=$(extract "$BODY" '.doc_id // empty')
BOOK_FRONT_PAGE_UID=$(extract "$BODY" '.uid // .doc_id // empty')
echo "       book_id=${BOOK_ID:-unknown}"

# ---------------------------------------------------------------------------
# 10. Book — get nav (chapters & sections)
# ---------------------------------------------------------------------------
echo ""
info "10. Book — read nav"

if [ -n "$BOOK_ID" ]; then
    RESP=$(api_req GET "/book/get/nav?doc_id=${BOOK_ID}")
    BODY=$(res_body "$RESP")
    assert_status "GET /book/get/nav?doc_id=$BOOK_ID" 200 "$RESP" > /dev/null
    BOOK_FRONT_PAGE_UID=$(extract "$BODY" '.main_node.uid // empty')
    echo "       front_page_uid=${BOOK_FRONT_PAGE_UID:-unknown}"
else
    skip "[book read nav] no book_id"
fi

# ---------------------------------------------------------------------------
# 11. Book — append chapter (identity=101)
# ---------------------------------------------------------------------------
echo ""
info "11. Book — append chapter"

if [ -n "$BOOK_ID" ] && [ -n "$BOOK_FRONT_PAGE_UID" ]; then
    RESP=$(api_req POST /book/append/node -d "{
        \"doc_id\": ${BOOK_ID},
        \"parent_id\": ${BOOK_FRONT_PAGE_UID},
        \"page_id\": ${BOOK_FRONT_PAGE_UID},
        \"title\": \"Chapter 1\",
        \"content\": \"<basic> Chapter 1 introduction.\",
        \"images\": [],
        \"identity\": 101,
        \"parent_identity\": 100,
        \"tags\": null
    }")
    BODY=$(res_body "$RESP")
    assert_status "POST /book/append/node (chapter)" 200 "$RESP" > /dev/null
    BOOK_CHAPTER_ID=$(extract "$BODY" '.new_node.uid // empty')
    echo "       chapter_id=${BOOK_CHAPTER_ID:-unknown}"
else
    skip "[book append chapter] missing book_id or front_page_uid"
fi

# ---------------------------------------------------------------------------
# 12. Book — append section (identity=102)
# ---------------------------------------------------------------------------
echo ""
info "12. Book — append section"

if [ -n "$BOOK_ID" ] && [ -n "$BOOK_CHAPTER_ID" ]; then
    RESP=$(api_req POST /book/append/node -d "{
        \"doc_id\": ${BOOK_ID},
        \"parent_id\": ${BOOK_CHAPTER_ID},
        \"page_id\": ${BOOK_CHAPTER_ID},
        \"title\": \"Section 1.1\",
        \"content\": \"<basic> Section content here.\",
        \"images\": [],
        \"identity\": 102,
        \"parent_identity\": 101,
        \"tags\": null
    }")
    BODY=$(res_body "$RESP")
    assert_status "POST /book/append/node (section)" 200 "$RESP" > /dev/null
    BOOK_SECTION_ID=$(extract "$BODY" '.new_node.uid // empty')
    echo "       section_id=${BOOK_SECTION_ID:-unknown}"
else
    skip "[book append section] missing chapter_id"
fi

# ---------------------------------------------------------------------------
# 13. Book — get chapter details
# ---------------------------------------------------------------------------
echo ""
info "13. Book — get chapter details"

if [ -n "$BOOK_ID" ] && [ -n "$BOOK_CHAPTER_ID" ]; then
    RESP=$(api_req GET "/book/get/chapter?doc_id=${BOOK_ID}&page_id=${BOOK_CHAPTER_ID}")
    assert_status "GET /book/get/chapter" 200 "$RESP" > /dev/null

    RESP=$(api_req GET "/book/get/section?doc_id=${BOOK_ID}&page_id=${BOOK_CHAPTER_ID}")
    assert_status "GET /book/get/section" 200 "$RESP" > /dev/null
else
    skip "[book get chapter/section] missing ids"
fi

# ---------------------------------------------------------------------------
# 14. Book — edit main
# ---------------------------------------------------------------------------
echo ""
info "14. Book — edit main"

if [ -n "$BOOK_ID" ] && [ -n "$BOOK_FRONT_PAGE_UID" ]; then
    RESP=$(api_req POST /book/edit/main -d "{
        \"doc_id\": ${BOOK_ID},
        \"uid\": ${BOOK_FRONT_PAGE_UID},
        \"title\": \"My First Book (Revised)\",
        \"content\": \"<basic> Updated introduction.\",
        \"images\": []
    }")
    assert_status "POST /book/edit/main" 200 "$RESP" > /dev/null
else
    skip "[book edit main] missing ids"
fi

# ---------------------------------------------------------------------------
# 15. Book — edit node
# ---------------------------------------------------------------------------
echo ""
info "15. Book — edit node"

if [ -n "$BOOK_CHAPTER_ID" ] && [ -n "$BOOK_ID" ]; then
    RESP=$(api_req POST /book/edit/node -d "{
        \"uid\": ${BOOK_CHAPTER_ID},
        \"doc_id\": ${BOOK_ID},
        \"title\": \"Chapter 1 (revised)\",
        \"content\": \"<basic> Revised chapter content.\",
        \"identity\": 101,
        \"images\": []
    }")
    assert_status "POST /book/edit/node" 200 "$RESP" > /dev/null
else
    skip "[book edit node] missing chapter_id"
fi

# ---------------------------------------------------------------------------
# 16. Book — delete node (section)
# ---------------------------------------------------------------------------
echo ""
info "16. Book — delete node (section)"

if [ -n "$BOOK_SECTION_ID" ] && [ -n "$BOOK_CHAPTER_ID" ]; then
    RESP=$(api_req POST /book/delete/node -d "{
        \"identity\": 102,
        \"delete_id\": ${BOOK_SECTION_ID},
        \"parent_id\": ${BOOK_CHAPTER_ID}
    }")
    assert_status "POST /book/delete/node (section)" 200 "$RESP" > /dev/null
else
    skip "[book delete section] missing section_id"
fi

# ---------------------------------------------------------------------------
# 17. User — follow / unfollow
# ---------------------------------------------------------------------------
echo ""
info "17. User — follow / unfollow"

if [ -n "$USER2_ID" ]; then
    # user1 follows user2
    RESP=$(api_req POST "/user/${USER2_ID}/subscribe")
    assert_status "POST /user/:id/subscribe (follow user2)" 200 "$RESP" > /dev/null

    # get subscribed list
    RESP=$(api_req GET /user/get_subscribed_users)
    assert_status "GET /user/get_subscribed_users" 200 "$RESP" > /dev/null

    # user1 unfollows user2
    RESP=$(api_req POST "/user/${USER2_ID}/un_subscribe")
    assert_status "POST /user/:id/un_subscribe (unfollow user2)" 200 "$RESP" > /dev/null
else
    skip "[follow/unfollow] user2_id not available"
fi

# User2 follows user1
if [ -n "$USER1_ID" ]; then
    RESP=$(api_req2 POST "/user/${USER1_ID}/subscribe")
    assert_status "POST /user/:id/subscribe (user2 follows user1)" 200 "$RESP" > /dev/null

    RESP=$(api_req2 GET /user/get_subscribed_users)
    assert_status "GET /user/get_subscribed_users (user2)" 200 "$RESP" > /dev/null

    RESP=$(api_req2 POST "/user/${USER1_ID}/un_subscribe")
    assert_status "POST /user/:id/un_subscribe (user2 unfollows user1)" 200 "$RESP" > /dev/null
fi

# ---------------------------------------------------------------------------
# 18. Cleanup — delete blog and book
# ---------------------------------------------------------------------------
echo ""
info "18. Cleanup — delete blog and book"

if [ -n "$BLOG_ID" ]; then
    RESP=$(api_req POST /blog/delete -d "{\"doc_id\": ${BLOG_ID}}")
    assert_status "POST /blog/delete" 200 "$RESP" > /dev/null
else
    skip "[blog delete] no blog_id"
fi

if [ -n "$BOOK_ID" ]; then
    RESP=$(api_req POST /book/delete -d "{\"doc_id\": ${BOOK_ID}}")
    assert_status "POST /book/delete" 200 "$RESP" > /dev/null
else
    skip "[book delete] no book_id"
fi

# ---------------------------------------------------------------------------
# 19. Auth — logout
# ---------------------------------------------------------------------------
echo ""
info "19. Logout"

RESP=$(auth_req POST /logout)
assert_status "POST /logout" 200 "$RESP" > /dev/null

# Protected endpoint should now 401
RESP=$(api_req POST /blog/create -d '{"title":"x","content":"x","images":[],"tags":[]}')
STATUS=$(res_status "$RESP")
if [ "$STATUS" = "401" ] || [ "$STATUS" = "403" ]; then
    pass "[unauthenticated create → 401/403]"
else
    fail "[unauthenticated create] expected 401/403, got $STATUS"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
TOTAL=$((PASS + FAIL))
echo -e "${CYAN}${BOLD}=== Results ===${NC}  $PASS/$TOTAL passed  ${YELLOW}($SKIP skipped)${NC}"
if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}${BOLD}$FAIL test(s) failed.${NC}"
    exit 1
fi
echo -e "${GREEN}${BOLD}All tests passed.${NC}"
