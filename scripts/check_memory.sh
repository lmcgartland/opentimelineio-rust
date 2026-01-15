#!/bin/bash
# Memory leak detection script using Valgrind
#
# Prerequisites:
#   macOS: brew install valgrind (note: limited support on recent macOS)
#   Linux: apt-get install valgrind / yum install valgrind
#
# Usage:
#   ./scripts/check_memory.sh           # Run all memory stress tests
#   ./scripts/check_memory.sh <test>    # Run specific test (e.g., stress_test_timeline_creation)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Memory Leak Detection ===${NC}"
echo ""

# Check if valgrind is available
if ! command -v valgrind &> /dev/null; then
    echo -e "${RED}Error: valgrind is not installed${NC}"
    echo ""
    echo "Install with:"
    echo "  macOS: brew install valgrind"
    echo "  Ubuntu/Debian: sudo apt-get install valgrind"
    echo "  RHEL/Fedora: sudo yum install valgrind"
    echo ""
    echo "Note: On recent macOS versions, you may need to use Docker with a Linux image,"
    echo "or use Instruments/Leaks instead."
    exit 1
fi

# Build the tests
echo -e "${YELLOW}Building tests...${NC}"
cargo build --tests 2>&1 | tail -5

# Find the memory test binary
TEST_BIN=$(find target/debug/deps -name 'memory-*' -type f -perm +111 2>/dev/null | head -1)

if [ -z "$TEST_BIN" ]; then
    # Try without perm flag (Linux compatibility)
    TEST_BIN=$(find target/debug/deps -name 'memory-*' -type f -executable 2>/dev/null | head -1)
fi

if [ -z "$TEST_BIN" ]; then
    echo -e "${RED}Error: Could not find memory test binary${NC}"
    echo "Make sure the tests compile correctly with: cargo test --test memory --no-run"
    exit 1
fi

echo -e "${GREEN}Found test binary: $TEST_BIN${NC}"
echo ""

# Determine which tests to run
if [ -n "$1" ]; then
    TEST_FILTER="$1"
    echo -e "${YELLOW}Running specific test: $TEST_FILTER${NC}"
else
    TEST_FILTER=""
    echo -e "${YELLOW}Running all memory stress tests...${NC}"
fi

echo ""
echo -e "${YELLOW}This may take several minutes...${NC}"
echo ""

# Run valgrind
# --leak-check=full: Show details for each definitely/possibly lost block
# --show-leak-kinds=all: Show all leak types (definite, indirect, possible, reachable)
# --track-origins=yes: Track origins of uninitialized values
# --error-exitcode=1: Exit with code 1 if errors found
# --suppressions: Add suppressions file if needed for known issues

VALGRIND_OPTS="--leak-check=full"
VALGRIND_OPTS="$VALGRIND_OPTS --show-leak-kinds=definite,indirect,possible"
VALGRIND_OPTS="$VALGRIND_OPTS --track-origins=yes"
VALGRIND_OPTS="$VALGRIND_OPTS --error-exitcode=1"
VALGRIND_OPTS="$VALGRIND_OPTS --errors-for-leak-kinds=definite"

# Add suppression file if it exists
SUPP_FILE="$(dirname "$0")/valgrind.supp"
if [ -f "$SUPP_FILE" ]; then
    VALGRIND_OPTS="$VALGRIND_OPTS --suppressions=$SUPP_FILE"
fi

# Run the tests
if [ -n "$TEST_FILTER" ]; then
    valgrind $VALGRIND_OPTS "$TEST_BIN" --ignored "$TEST_FILTER" --test-threads=1 --nocapture
else
    valgrind $VALGRIND_OPTS "$TEST_BIN" --ignored --test-threads=1
fi

RESULT=$?

echo ""
if [ $RESULT -eq 0 ]; then
    echo -e "${GREEN}=== Memory check PASSED ===${NC}"
    echo "No definite memory leaks detected."
else
    echo -e "${RED}=== Memory check FAILED ===${NC}"
    echo "Memory leaks or errors were detected."
    echo "Review the valgrind output above for details."
fi

exit $RESULT
