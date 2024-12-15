import {
  SearchSelect,
  SearchSelectItem,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeaderCell,
  TableRow,
} from "@tremor/react";
import { SpendPerMonth } from "../bindings/SpendPerMonth";
import { useEffect, useState } from "react";
import { Account } from "../bindings/Account";
import { API_URL } from "../main";
import { ListLedgerResponse } from "../bindings/ListLedgerResponse";
import { LedgerMeta } from "../bindings/LedgerMeta";

export function SpendBreakdownTransactions({
  presets,
}: {
  spendPerMonth: SpendPerMonth;
  presets: Preset[];
}) {
  const [ledgers, setLedgers] = useState([] as LedgerMeta[]);
  const [ledger, setLedger] = useState(undefined as LedgerMeta | undefined);
  const [data, setData] = useState(undefined as Account | undefined);

  useEffect(() => {
    const load = async () => {
      const response = await fetch(`${API_URL}/ledgers`, {
        credentials: "include",
        redirect: "follow",
      });
      const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;
      setLedgers(ledgers);
      setLedger(ledgers.length > 0 ? ledgers[0] : undefined);
    };
    load();
  }, [presets]);

  useEffect(() => {
    const load = async (ledger: LedgerMeta) => {
      const year = presets[0].year;
      const month = presets[0].month;
      const from = `${year}-${month}-01`;
      const to = `${year}-${month}-${daysInMonth(month, year)}`;
      const response = await fetch(
        `${API_URL}/ledger/${ledger.id}?from=${from}&to=${to}`,
        {
          credentials: "include",
          redirect: "follow",
        }
      );
      const data = (await response.json()) as Account;
      setData(data);
    };
    if (ledger) {
      load(ledger);
    }
  }, [presets, ledger]);

  return (
    <>
      <SearchSelect
        className="my-2"
        onValueChange={(v) => setLedger(ledgers.find((l) => l.id == v))}
      >
        {ledgers.map((l) => (
          <SearchSelectItem key={l.id} value={l.id}>
            {l.name}
          </SearchSelectItem>
        ))}
      </SearchSelect>
      {data && (
        <Table className="mt-5">
          <TableHead>
            <TableRow>
              <TableHeaderCell className="text-right">Date</TableHeaderCell>
              <TableHeaderCell className="text-right">Amount</TableHeaderCell>
              <TableHeaderCell className="text-right">Category</TableHeaderCell>
              <TableHeaderCell className="text-right">
                Description
              </TableHeaderCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {data.ledgers.map((ledger) => (
              <>
                <TableRow>
                  <TableCell colSpan={4}>
                    <h1 className="text-4xl">{ledger.symbol}</h1>
                  </TableCell>
                </TableRow>
                {ledger.records.map((item, i) => (
                  <TableRow key={i}>
                    <TableCell className="text-right">
                      {new Date(item.date).toDateString()}
                    </TableCell>
                    <TableCell className="text-right">
                      {item.amount?.toFixed(2)}
                    </TableCell>
                    <TableCell className="text-right">
                      {item.description}
                    </TableCell>
                    <TableCell className="text-right">
                      {item.category}
                    </TableCell>
                  </TableRow>
                ))}
              </>
            ))}
          </TableBody>
        </Table>
      )}
    </>
  );
}

export interface Preset {
  month: number;
  year: number;
}

// Month in JavaScript is 0-indexed (January is 0, February is 1, etc),
// but by using 0 as the day it will give us the last day of the prior
// month. So passing in 1 as the month number will return the last day
// of January, not February
function daysInMonth(month: number, year: number) {
  return new Date(year, month, 0).getDate();
}
