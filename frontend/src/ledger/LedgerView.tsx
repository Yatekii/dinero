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
  let response = await fetch(`${API_URL}/ledgers`);
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;
  let data = undefined;
  if (params.ledgerId != undefined) {
    response = await fetch(`${API_URL}/ledger/${params.ledgerId}`);
    data = (await response.json()) as Account;
  }

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
      <Table className="mt-5">
        <TableHead>
          <TableRow>
            <TableHeaderCell className="text-right">Date</TableHeaderCell>
            <TableHeaderCell className="text-right">Amount</TableHeaderCell>
            <TableHeaderCell className="text-right">
              Description
            </TableHeaderCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {currentLedger.records.map((item, i) => (
            <TableRow key={i}>
              <TableCell className="text-right">
                {new Date(item.date).toDateString()}
              </TableCell>
              <TableCell className="text-right">
                {item.amount.toFixed(2)}
              </TableCell>
              <TableCell className="text-right">{item.description}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </>
  );
}
