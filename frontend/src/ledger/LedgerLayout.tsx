import { Button, Card, Tab, TabGroup, TabList } from "@tremor/react";
import {
  Outlet,
  useLoaderData,
  useNavigate,
  useParams,
} from "react-router-dom";
import { ListLedgerResponse } from "../bindings/ListLedgerResponse";
import { CakeIcon } from "@heroicons/react/24/solid";

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgersLoader() {
  const response = await fetch("http://127.0.0.1:3000/ledgers");
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;

  return { ledgers };
}

export function LedgerLayout() {
  return (
    <Card className="mx-full">
      <LedgerMenu />
      <Outlet />
    </Card>
  );
}

function LedgerMenu() {
  const { ledgers } = useLoaderData() as Awaited<
    ReturnType<typeof ledgersLoader>
  >;
  const navigate = useNavigate();
  const params = useParams();

  return (
    <div className="flex items-center mb-3">
      <TabGroup
        index={
          params.ledgerId
            ? ledgers.findIndex((l) => l.id == params.ledgerId) + 1
            : 0
        }
        onIndexChange={(index) =>
          index == 0
            ? navigate(`/ledger`)
            : navigate(`/ledger/${ledgers[index - 1].id}`)
        }
      >
        <TabList variant="solid" className="flex flex-wrap p-2">
          <>
            <Tab icon={CakeIcon}>"All"</Tab>
            {ledgers.map((l) => (
              <Tab key={l.name + l.currency} icon={CakeIcon}>
                {l.name} [{l.currency}]
              </Tab>
            ))}
          </>
        </TabList>
      </TabGroup>
      <Button
        variant="primary"
        onClick={() => navigate(`/ledger/add`)}
        size="sm"
        className="ml-2 py-1 px-3"
      >
        Add
      </Button>
    </div>
  );
}
