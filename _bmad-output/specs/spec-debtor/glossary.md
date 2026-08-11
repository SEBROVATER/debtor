# Debtor Glossary

- **Administrator:** The single person authenticated to operate Debtor.
- **Group:** A private ledger that owns Participants, Spendings, one Group Currency, current-month summaries, Balances, and Settlement Transfers.
- **Participant:** A Group-owned accounting identity, not an application user and not reusable across Groups.
- **Spending:** A dated transaction with a positive Total, one Source Currency, one category, exactly one Payer, and Participant Shares.
- **Payer:** The Participant who paid a Spending's Total.
- **Share:** The exact portion of a Spending Total attributed to a Participant.
- **Source Currency:** The original currency retained by a Spending.
- **Group Currency:** The Group-selected currency used for converted summaries, Balances, and Settlement Transfers.
- **Current-Month Summary:** Spending totals for the selected Group whose dates fall in the current UTC calendar month.
- **Balance:** A Participant's all-time net position in Group Currency, derived on demand from all Group Spendings.
- **Settlement Transfer:** An advisory payment between Participants that would settle all-time Balances; it is not a recorded repayment.
