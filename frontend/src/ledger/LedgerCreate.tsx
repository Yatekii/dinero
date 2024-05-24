import { Button, Select, SelectItem, Textarea, TextInput } from "@tremor/react";
import { CalculatorIcon } from "@heroicons/react/24/solid";
import { useState } from "react";

export function LedgerCreate() {
  const [content, setContent] = useState("");
  const [format, setFormat] = useState("neon");
  const [name, setName] = useState("");
  const [initialBalance, setInitialBalance] = useState("");
  const [currency, setCurrency] = useState("CHF");

  return (
    <>
      <TextInput
        className="mt-5"
        placeholder="Name"
        value={name}
        onValueChange={setName}
      />
      <TextInput
        className="mt-5"
        placeholder="Initial Balance"
        value={initialBalance}
        onValueChange={setInitialBalance}
      />
      <Select value={currency} onValueChange={setCurrency} className="mt-5">
        <SelectItem value="CHF" icon={CalculatorIcon}>
          CHF
        </SelectItem>
        <SelectItem value="USD" icon={CalculatorIcon}>
          USD
        </SelectItem>
        <SelectItem value="GBP" icon={CalculatorIcon}>
          GBP
        </SelectItem>
        <SelectItem value="PLN" icon={CalculatorIcon}>
          PLN
        </SelectItem>
        <SelectItem value="EUR" icon={CalculatorIcon}>
          EUR
        </SelectItem>
      </Select>
      <Textarea
        className="mt-5"
        onChange={(e) => setContent(e.target.value)}
        id="description"
        placeholder="Paste your CSV here..."
        value={content}
      />
      <Select value={format} onValueChange={setFormat} className="mt-5">
        <SelectItem value="neon" icon={CalculatorIcon}>
          Neon
        </SelectItem>
        <SelectItem value="ubs" icon={CalculatorIcon}>
          UBS
        </SelectItem>
      </Select>

      <Button
        size="sm"
        className="mt-5"
        onClick={() => {
          fetch(`http://127.0.0.1:3000/ledger`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              transactions_data: content,
              name,
              initialBalance: +initialBalance,
              currency,
              format,
            }),
          });
        }}
      >
        Import
      </Button>
    </>
  );
}
