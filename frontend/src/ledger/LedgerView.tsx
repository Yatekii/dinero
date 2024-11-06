import {
  Button,
  Card,
  Dialog,
  DialogPanel,
  Flex,
  Title,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeaderCell,
  TableRow,
  Textarea,
} from "@tremor/react";
import { FolderOpenIcon } from "@heroicons/react/24/solid";
import { Navigate, useLoaderData } from "react-router-dom";
import { useState } from "react";
import type { ListLedgerResponse } from "../bindings/ListLedgerResponse";
import { Account } from "../bindings/Account";

export type Params<Key extends string = string> = {
  readonly [key in Key]: string | undefined;
};

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgerLoader({ params }: { params: Params }) {
  let response = await fetch("http://127.0.0.1:3000/ledgers");
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;
  let data = undefined;
  if (params.ledgerId != undefined) {
    response = await fetch(`http://127.0.0.1:3000/ledger/${params.ledgerId}`);
    data = (await response.json()) as Account;
  }

  return { ledgers, currentLedger: data };
}

export function Ledger() {
  const { ledgers, currentLedger } = useLoaderData() as Awaited<
    ReturnType<typeof ledgerLoader>
  >;
  const [isImportOpen, setIsImportOpen] = useState(false);

  if (currentLedger == undefined) {
    if (ledgers.length > 0) {
      return <Navigate to={`/ledger/${ledgers[0].id}`} />;
    } else {
      return <p>No ledgers found</p>;
    }
  }

  return (
    <>
      <Card>
        <div className="flex">
          <Title>Ledgers</Title>
          <Flex justifyContent="end">
            <Button
              size="sm"
              className="py-1 h-8"
              icon={FolderOpenIcon}
              onClick={() => {
                setIsImportOpen(true);
              }}
            >
              Import transactions
            </Button>
            <Dialog
              open={isImportOpen}
              onClose={(val) => setIsImportOpen(val)}
              static={true}
            >
              <DialogPanel className="overflow-visible">
                <Title className="mb-3">
                  <Flex>
                    Import new transactions to the ledger
                    <Button
                      variant="light"
                      onClick={() => setIsImportOpen(false)}
                    >
                      Close
                    </Button>
                  </Flex>
                </Title>
                <Import ledgerId={currentLedger.id} />
              </DialogPanel>
            </Dialog>
          </Flex>
        </div>
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
      </Card>
    </>
  );
}

function Import({ ledgerId }: { ledgerId: string }) {
  const [content, setContent] = useState("");
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  return (
    <>
      <Textarea
        className="mt-5"
        onChange={(e) => setContent(e.target.value)}
        id="description"
        placeholder="Paste your CSV here..."
        value={content}
      />
      <Button
        size="sm"
        className="mt-5"
        onClick={async () => {
          setError("");
          const response = await fetch(
            `http://127.0.0.1:3000/ledger/${ledgerId}`,
            {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                transactions_data: content,
              }),
            }
          );

          if (response.ok) {
            setSuccess("Values added!");
            setTimeout(() => setSuccess(""), 3000);
          } else {
            setError(await response.text());
          }
        }}
      >
        Import
      </Button>
      {error && <p className="text-red">{error}</p>}
      {success && <p className="text-green">{success}</p>}
    </>
  );
}
