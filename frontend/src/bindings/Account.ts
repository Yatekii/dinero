// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { BankFormat } from "./BankFormat";
import type { Currency } from "./Currency";
import type { ExtendedLedger } from "./ExtendedLedger";
import type { Owner } from "./Owner";

export type Account = { id: string, 
/**
 * The OIDC owner
 */
owner: Owner, name: string, currency: Currency, format: BankFormat, ledgers: Array<ExtendedLedger>, initial_balance: number | null, initial_date: number, spending: boolean, };