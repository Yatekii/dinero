import { Button, Select, SelectItem, TextInput, Title } from "@tremor/react";
import { useState } from "react";
import { CreateLedgerRequest } from "../bindings/CreateLedgerRequest";
import { Checkbox } from "../components/Checkbox";
import { useNavigate } from "react-router-dom";
import { CreateLedgerResponse } from "../bindings/CreateLedgerResponse";
import { BANKS, CURRENCIES } from "../lib/currency";

export function LedgerCreate() {
  const navigate = useNavigate();
  const [format, setFormat] = useState("neon");
  const [name, setName] = useState("");
  const [initialBalance, setInitialBalance] = useState("");
  const [initialDate, setInitialDate] = useState(
    "" + new Date().toDateString()
  );
  const [hasInitialDate, setHasInitialDate] = useState(
    false as boolean | "indeterminate"
  );
  const [currency, setCurrency] = useState("CHF");

  return (
    <>
      <Title className="mb-2">Add a new ledger</Title>
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
      <Select value={format} onValueChange={setFormat} className="mt-5">
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
          const response = await fetch(`http://127.0.0.1:3000/ledger`, {
            method: "POST",
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
          });
          if (response.status == 200) {
            const data = (await response.json()) as CreateLedgerResponse;
            navigate(`http://127.0.0.1:3000/ledger/${data.id}`);
          }
        }}
      >
        Create Ledger
      </Button>
    </>
  );
}
