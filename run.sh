#!/bin/bash

# CtrlQ - Easy runner script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎯 CtrlQ - Developer Keylogger${NC}"
echo -e "${YELLOW}⚠️  This tool requires root privileges to access keyboard devices${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}❌ Please run with sudo:${NC}"
    echo -e "   ${GREEN}sudo $0 $@${NC}"
    exit 1
fi

# Build if needed
if [ ! -f "target/release/ctrlq" ]; then
    echo -e "${YELLOW}🔧 Building CtrlQ...${NC}"
    cargo build --release
fi

# Parse arguments
case "${1:-}" in
    "list"|"--list"|"-l"|"--list-devices")
        echo -e "${BLUE}🔍 Listing keyboard devices...${NC}"
        ./target/release/ctrlq --list-devices
        ;;
    "help"|"--help"|"-h")
        ./target/release/ctrlq --help
        ;;
    *)
        echo -e "${GREEN}🚀 Starting CtrlQ with UI...${NC}"
        echo -e "${YELLOW}💡 Press 'q' to quit, 'Tab' to switch tabs${NC}"
        echo ""
        ./target/release/ctrlq "$@"
        ;;
esac
