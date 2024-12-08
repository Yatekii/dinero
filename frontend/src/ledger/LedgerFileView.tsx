import {
  Button,
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
import {
  CheckIcon,
  CloudArrowUpIcon,
  TrashIcon,
  XMarkIcon,
} from "@heroicons/react/24/solid";
import { Navigate, useLoaderData, useNavigate } from "react-router-dom";
import { useCallback, useState } from "react";
import type { ListLedgerResponse } from "../bindings/ListLedgerResponse";
import { LedgerFiles } from "../bindings/LedgerFiles";
import { Tooltip } from "../components/Tooltip";
import Dropzone, { useDropzone } from "react-dropzone";
import { API_URL } from "../main";

export type Params<Key extends string = string> = {
  readonly [key in Key]: string | undefined;
};

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgerFileLoader({ params }: { params: Params }) {
  let response = await fetch(`${API_URL}/ledgers`);
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;
  let data = undefined;
  if (params.ledgerId != undefined) {
    response = await fetch(`${API_URL}/ledger/${params.ledgerId}/files`);
    data = (await response.json()) as LedgerFiles;
  }

  return { ledgers, currentLedger: data, ledgerId: params.ledgerId! };
}

export function LedgerFileView() {
  const { ledgers, currentLedger, ledgerId } = useLoaderData() as Awaited<
    ReturnType<typeof ledgerFileLoader>
  >;
  const [isImportOpen, setIsImportOpen] = useState(false);
  const navigate = useNavigate();

  const updateFile = async (id: string, name: string, file: File) => {
    const body = new FormData();
    body.append("file", file);

    await fetch(`${API_URL}/ledger/${id}/files/${name}`, {
      method: "PUT",
      body,
    });

    navigate(`/ledger/${id}/files`);
  };
  const addFiles = async (id: string, files: File[]) => {
    const body = new FormData();
    body.append(`${files[0].name}`, files[0]);

    await fetch(`${API_URL}/ledger/${id}/files`, {
      method: "POST",
      body,
    });

    navigate(`/ledger/${id}/files`);
  };

  const deleteFile = async (id: string, name: string) => {
    await alert(`Are you sure you want to delete the file ${name}?`);
    await fetch(`${API_URL}/api/ledger/${id}/files/${name}`, {
      method: "DELETE",
    });

    navigate(`/ledger/${id}/files`);
  };

  const onDrop = useCallback(
    (files: File[]) => {
      addFiles(ledgerId, files);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ledgerId]
  );
  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop });

  if (currentLedger == undefined) {
    if (ledgers.length > 0) {
      return <Navigate to={`/ledger/${ledgers[0].id}`} />;
    } else {
      return <p>No ledgers found</p>;
    }
  }

  return (
    <>
      <div className="flex">
        <Flex justifyContent="end">
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
            <TableHeaderCell className="text-left">File Name</TableHeaderCell>
            <TableHeaderCell className="text-left">Entries</TableHeaderCell>
            <TableHeaderCell className="text-right">Status</TableHeaderCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {currentLedger.files.map((item, i) => (
            <Dropzone
              key={i}
              onDrop={(acceptedFiles) =>
                updateFile(ledgerId, item.filename, acceptedFiles[0])
              }
            >
              {({ getRootProps, getInputProps, isDragActive }) => (
                <TableRow key={i}>
                  <>
                    <TableCell {...getRootProps()} className="text-left">
                      <input {...getInputProps()} className="hidden" />
                      {!isDragActive
                        ? item.filename
                        : "Drop the files here ..."}
                    </TableCell>
                    <TableCell className="text-left">
                      {item.number_of_entries}
                    </TableCell>
                    <TableCell className="flex justify-end">
                      {item.error ? (
                        <Tooltip side="top" content={item.error}>
                          <XMarkIcon className="py-1 mr-2 h-6 text-red-700" />
                        </Tooltip>
                      ) : (
                        <>
                          <Tooltip
                            side="top"
                            content="Imported file parses correctly."
                          >
                            <CheckIcon className="py-1 mr-2 h-6 text-green-700" />
                          </Tooltip>
                        </>
                      )}
                      <Tooltip
                        side="top"
                        content="Imported file parses correctly."
                      >
                        <TrashIcon
                          className="py-1 h-6 text-white"
                          onClick={() => deleteFile(ledgerId, item.filename)}
                        />
                      </Tooltip>
                    </TableCell>
                  </>
                </TableRow>
              )}
            </Dropzone>
          ))}
        </TableBody>
      </Table>
      <div
        {...getRootProps()}
        className="mt-2 p-2 h-24 border-dashed border-2 border-gray-600 text-gray-600 rounded-md flex items-center justify-center"
      >
        <input {...getInputProps()} />
        {isDragActive ? (
          <p className="text-center">Drop the files here ...</p>
        ) : (
          <p className="text-center">
            <CloudArrowUpIcon className="w-5 inline-block -mt-1" /> Drag 'n'
            drop some files here, or click to select files
          </p>
        )}
      </div>
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
          const response = await fetch(`${API_URL}/ledger/${ledgerId}`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              transactions_data: content,
            }),
          });

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
