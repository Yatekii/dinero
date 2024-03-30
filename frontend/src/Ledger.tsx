import {
  Button,
  Card,
  Dialog,
  DialogPanel,
  Flex,
  Select,
  SelectItem,
  Tab,
  TabGroup,
  TabList,
  Title,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeaderCell,
  TableRow,
  Textarea,
  TextInput,
} from "@tremor/react";
import {
  CakeIcon,
  FolderOpenIcon,
  CalculatorIcon,
} from "@heroicons/react/24/solid";
import { useLoaderData, useNavigate } from "react-router-dom";
import { useState } from "react";

export async function ledgerLoader({ params }) {
  let response = await fetch("http://127.0.0.1:3000/ledgers");
  const ledgers = (await response.json()).ledgers;
  response = await fetch(`http://127.0.0.1:3000/ledger/${params.ledgerId}`);
  const data = await response.json();
  return { ledgers, currentLedger: data };
}

export function Ledger() {
  const { ledgers, currentLedger } = useLoaderData();
  const navigate = useNavigate();
  const [isImportOpen, setIsImportOpen] = useState(true);

  return (
    <>
      <Card>
        <Title>Ledgers</Title>
        <TabGroup
          index={ledgers.findIndex((l) => l.id == currentLedger.id)}
          onIndexChange={(index) => navigate(`/ledger/${ledgers[index].id}`)}
        >
          <TabList variant="solid" className="mt-8">
            {ledgers.map((l) => (
              <Tab key={l.name + l.currency} icon={CakeIcon}>
                {l.name} [{l.currency}]
              </Tab>
            ))}
          </TabList>
        </TabGroup>
        <div className="mt-10">
          <Flex justifyContent="end">
            <Button
              size="sm"
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
          <Table className="mt-5">
            <TableHead>
              <TableRow>
                <TableHeaderCell className="text-right">Date</TableHeaderCell>
                <TableHeaderCell className="text-right">Amount</TableHeaderCell>
                <TableHeaderCell className="text-right">Total</TableHeaderCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {[
                ...Array(
                  currentLedger.transactions.df.columns[0].values.length
                ).keys(),
              ]
                .map((i) => [
                  currentLedger.transactions.df.columns[0].values[i],
                  currentLedger.transactions.df.columns[1].values[i],
                  currentLedger.transactions.df.columns[2].values[i],
                ])
                .map((item, i) => (
                  <TableRow key={i}>
                    <TableCell className="text-right">
                      {new Date(item[0] * 24 * 60 * 60 * 1000).toDateString()}
                    </TableCell>
                    <TableCell className="text-right">
                      {item[1]?.toFixed(2)}
                    </TableCell>
                    <TableCell className="text-right">
                      {item[2]?.toFixed(2)}
                    </TableCell>
                  </TableRow>
                ))}
            </TableBody>
          </Table>
        </div>
      </Card>
    </>
  );
}

function Import({ ledgerId }) {
  const [content, setContent] = useState("");
  const [format, setFormat] = useState("");
  const [id, setId] = useState("");
  const [name, setName] = useState("");
  const [currency, setCurrency] = useState("");
  return (
    <>
      <TextInput
        className="mt-5"
        placeholder="ID"
        value={id}
        onValueChange={setId}
      />
      <TextInput
        className="mt-5"
        placeholder="Name"
        value={name}
        onValueChange={setName}
      />
      <TextInput
        className="mt-5"
        placeholder="Currency"
        value={currency}
        onValueChange={setCurrency}
      />
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
          fetch(`http://127.0.0.1:3000/ledger/${ledgerId}`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              transactions_data: content,
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
