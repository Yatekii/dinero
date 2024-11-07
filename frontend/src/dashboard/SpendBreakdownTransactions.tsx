import {
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

export function SpendBreakdownTransactions({
  presets,
}: {
  spendPerMonth: SpendPerMonth;
  presets: Preset[];
}) {
  const [data, setData] = useState(undefined as Account | undefined);

  useEffect(() => {
    const load = async () => {
      const year = presets[0].year;
      const month = presets[0].month;
      const from = `${year}-${month}-01`;
      const to = `${year}-${month}-${daysInMonth(month, year)}`;
      const response = await fetch(
        `http://127.0.0.1:3000/ledger/neon?from=${from}&to=${to}`
      );
      const data = (await response.json()) as Account;
      setData(data);
    };
    load();
  }, [presets]);

  return (
    <>
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
            {data.records.map((item, i) => (
              <TableRow key={i}>
                <TableCell className="text-right">
                  {new Date(item.date).toDateString()}
                </TableCell>
                <TableCell className="text-right">
                  {item.amount?.toFixed(2)}
                </TableCell>
                <TableCell className="text-right">{item.description}</TableCell>
                <TableCell className="text-right">{item.category}</TableCell>
              </TableRow>
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
