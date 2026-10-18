#!/usr/bin/env python3
"""A still picture of a Claude Code session for the demo cockpit's session pane: a made-up
conversation on the demo's acme-shop checkout, drawn once, then it waits."""

import shutil
import time

BOLD, DIM, RESET = "\033[1m", "\033[2m", "\033[0m"
WHITE, GREEN, RED, BLUE, ORANGE, GREY = "\033[97m", "\033[32m", "\033[31m", "\033[34m", "\033[38;5;173m", "\033[90m"

CONVERSATION = f"""
{GREY}>{RESET} Add VAT to the checkout total, per the customer's country. Rates are in config/vat.json.

{WHITE}●{RESET} I'll read how the total is computed, then add the VAT step and its tests.

{GREEN}●{RESET} {BOLD}Read{RESET}(src/checkout/total.ts)
  {GREY}⎿  Read 48 lines{RESET}

{GREEN}●{RESET} {BOLD}Update{RESET}(src/checkout/total.ts)
  {GREY}⎿  Updated src/checkout/total.ts with 7 additions and 2 removals{RESET}
       {GREY}21{RESET} {RED}-  return subtotal + shipping{RESET}
       {GREY}21{RESET} {GREEN}+  const rate = vatRate(customer.country){RESET}
       {GREY}22{RESET} {GREEN}+  const vat = roundCents((subtotal + shipping) * rate){RESET}
       {GREY}23{RESET} {GREEN}+  return {{ net: subtotal + shipping, vat, gross: subtotal + shipping + vat }}{RESET}

{GREEN}●{RESET} {BOLD}Bash{RESET}(npm test -- checkout)
  {GREY}⎿  PASS  tests/checkout/total.test.ts (9 tests){RESET}
     {GREY}PASS  tests/checkout/vat.test.ts (6 tests){RESET}

{WHITE}●{RESET} Checkout totals now include VAT for the customer's country:
  - {BOLD}total.ts{RESET} returns net, VAT and gross, VAT rounded to the cent
  - {BOLD}vat.test.ts{RESET} covers France, Germany, a zero-rate country and a missing rate
  - The product page still shows net prices: that is the next todo item.

{ORANGE}✻{RESET} {ORANGE}Writing the pull request description…{RESET} {GREY}(38s · ↓ 1.2k tokens){RESET}
"""


def main():
    columns = shutil.get_terminal_size().columns
    rule = GREY + "─" * columns + RESET
    print("\033[2J\033[H", end="")
    print(CONVERSATION)
    print(rule)
    print(f"{GREY}>{RESET} ")
    print(rule)
    print(f"  {GREY}⏵⏵ auto mode on (shift+tab to cycle){RESET}", end="", flush=True)
    while True:
        time.sleep(3600)


if __name__ == "__main__":
    main()
