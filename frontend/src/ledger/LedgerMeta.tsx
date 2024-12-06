import { Button, Select, SelectItem, TextInput, Title } from "@tremor/react";
import { useState } from "react";
import { CreateLedgerRequest } from "../bindings/CreateLedgerRequest";
import { Checkbox } from "../components/Checkbox";
import { useLoaderData, useNavigate } from "react-router-dom";
import { CreateLedgerResponse } from "../bindings/CreateLedgerResponse";
import { Account } from "../bindings/Account";
import { BANKS, CURRENCIES } from "../lib/currency";

export type Params<Key extends string = string> = {
  readonly [key in Key]: string | undefined;
};

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgerMetaLoader({ params }: { params: Params }) {
  let currentLedger;
  if (params.ledgerId != undefined) {
    const response = await fetch(
      `http://127.0.0.1:3000/ledger/${params.ledgerId}`
    );
    currentLedger = (await response.json()) as Account;
  }

  return { currentLedger };
}

export function LedgerMetaView() {
  const navigate = useNavigate();

  const { currentLedger } = useLoaderData() as Awaited<
    ReturnType<typeof ledgerMetaLoader>
  >;

  const [format, setFormat] = useState(currentLedger?.format ?? "neon");
  const [name, setName] = useState(currentLedger?.name);
  const [initialBalance, setInitialBalance] = useState(
    "" + (currentLedger?.initial_balance ?? "")
  );
  const [initialDate, setInitialDate] = useState(
    "" + (currentLedger?.initial_date ?? "" + new Date())
  );
  const [hasInitialDate, setHasInitialDate] = useState(
    (currentLedger?.initial_balance != null) as boolean | "indeterminate"
  );
  const [currency, setCurrency] = useState(currentLedger?.currency ?? "Chf");

  if (currentLedger == undefined) {
    return <p>No ledger found</p>;
  }

  return (
    <>
      <Title className="mb-2">Change {currentLedger.id}</Title>
      <TextInput placeholder="Name" value={name} onValueChange={setName} />
      <div className="flex py-0 mt-5">
        <Checkbox
          checked={hasInitialDate}
          onCheckedChange={setHasInitialDate}
          className="mt-1 mr-1"
        />
        <label className="text-gray-600">
          Transactions start not from 0 (add an initial balance)
        </label>
      </div>
      {hasInitialDate && (
        <>
          <TextInput
            className="mt-5"
            placeholder="Initial Balance"
            value={initialBalance}
            onValueChange={setInitialBalance}
          />
          <TextInput
            className="mt-5"
            placeholder="Initial Date"
            value={initialDate}
            onValueChange={setInitialDate}
          />
        </>
      )}
      <Select value={currency} onValueChange={setCurrency} className="mt-5">
        {CURRENCIES.map((v) => (
          <SelectItem key={v.symbol} value={v.symbol} icon={v.icon}>
            {v.symbol} - {v.name}
          </SelectItem>
        ))}
      </Select>
      <Select
        value={format}
        onValueChange={(v) => setFormat(v as "neon")}
        className="mt-5"
      >
        {BANKS.map((v) => (
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          <SelectItem key={v.id} value={v.id} icon={v.icon as any}>
            {v.name}
          </SelectItem>
        ))}
      </Select>

      <Button
        size="sm"
        className="mt-5"
        onClick={async () => {
          const response = await fetch(
            `http://127.0.0.1:3000/ledger/${currentLedger.id}`,
            {
              method: "PUT",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                name,
                format,
                initialBalance: hasInitialDate ? +initialBalance : undefined,
                initialDate: hasInitialDate ? +initialDate : undefined,
                currency,
                spending: false,
              } as CreateLedgerRequest),
            }
          );
          if (response.status == 200) {
            const data = (await response.json()) as CreateLedgerResponse;
            navigate(`/ledger/${data.id}/meta`);
          }
        }}
      >
        Update Ledger
      </Button>
      <Button
        size="sm"
        className="mt-5 ml-3 dark:bg-red-600 dark:border-red-600 dark:hover:bg-red-500 dark:hover:border-red-500"
        onClick={async () => {
          await alert(
            `Are you sure you want to delete ledger ${currentLedger.id}`
          );
          const response = await fetch(
            `http://127.0.0.1:3000/ledger/${currentLedger.id}`,
            {
              method: "DELETE",
            }
          );
          if (response.status == 200) {
            navigate(`/ledger/`);
          }
        }}
      >
        Delete Ledger
      </Button>
    </>
  );
}
