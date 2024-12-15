import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeaderCell,
  TableRow,
} from "@tremor/react";
import { Navigate, useLoaderData } from "react-router-dom";
import type { ListLedgerResponse } from "../bindings/ListLedgerResponse";
import { Account } from "../bindings/Account";
import { API_URL } from "../main";

export type Params<Key extends string = string> = {
  readonly [key in Key]: string | undefined;
};

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgerLoader({ params }: { params: Params }) {
  let response = await fetch(`${API_URL}/ledgers`, {
    credentials: "include",
    redirect: "follow",
  });
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;
  let data = undefined;
  if (params.ledgerId != undefined) {
    response = await fetch(`${API_URL}/ledger/${params.ledgerId}`, {
      credentials: "include",
      redirect: "follow",
    });
    data = (await response.json()) as Account;
  }

  data?.ledgers.sort(
    (a, b) => b.kind.localeCompare(a.kind) || a.symbol.localeCompare(b.symbol)
  );

  return { ledgers, currentLedger: data };
}

export function Ledger() {
  const { ledgers, currentLedger } = useLoaderData() as Awaited<
    ReturnType<typeof ledgerLoader>
  >;

  if (currentLedger == undefined) {
    if (ledgers.length > 0) {
      return <Navigate to={`/ledger/${ledgers[0].id}`} />;
    } else {
      return <p>No ledgers found</p>;
    }
  }

  return (
    <>
      {currentLedger.ledgers.map((ledger) => (
        <div key={ledger.symbol} className="mt-5 mx-3">
          <h1 className="text-4xl text-white">{ledger.symbol}</h1>
          <Table className="w-full mt-5">
            <TableHead>
              <TableRow>
                <TableHeaderCell className="text-right">
                  Description
                </TableHeaderCell>
                <TableHeaderCell className="text-right">Date</TableHeaderCell>
                <TableHeaderCell className="text-right">Amount</TableHeaderCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {ledger.records.map((item, i) => (
                <TableRow key={i}>
                  <TableCell className="text-right">
                    {item.description}
                  </TableCell>
                  <TableCell className="text-right">
                    {new Date(item.date).toDateString()}
                  </TableCell>
                  <TableCell className="text-right">
                    {item.amount.toFixed(2)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ))}
    </>
  );
}
