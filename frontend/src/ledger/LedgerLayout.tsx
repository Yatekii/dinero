import { Button, Tab, TabGroup, TabList } from "@tremor/react";
import {
  Outlet,
  useLoaderData,
  useLocation,
  useNavigate,
  useParams,
} from "react-router-dom";
import { ListLedgerResponse } from "../bindings/ListLedgerResponse";
import { CakeIcon } from "@heroicons/react/24/solid";
import { API_URL } from "../main";

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgersLoader() {
  const response = await fetch(`${API_URL}/ledgers`, {
    credentials: "include",
    redirect: "follow",
  });
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;

  return { ledgers };
}

export function LedgerLayout() {
  const params = useParams();
  return (
    <div className="mx-full flex flex-row overflow-scroll h-screen">
      <LedgerSelector />
      <div className="w-full">
        {params.ledgerId && <LedgerMenu />}
        <Outlet />
      </div>
    </div>
  );
}

function LedgerSelector() {
  const { ledgers } = useLoaderData() as Awaited<
    ReturnType<typeof ledgersLoader>
  >;
  const navigate = useNavigate();
  const params = useParams();

  return (
    <>
      <div className="flex flex-col mb-5">
        <Button
          variant="primary"
          onClick={() => navigate(`/ledger/add`)}
          className="py-1 px-3 mb-5"
        >
          Add
        </Button>
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
          <TabList variant="solid" className="flex flex-wrap p-2 flex-col">
            <>
              <Tab icon={CakeIcon}>All</Tab>
              {ledgers.map((l) => (
                <Tab key={l.name} icon={CakeIcon} style={{ marginLeft: 0 }}>
                  {l.name}
                </Tab>
              ))}
            </>
          </TabList>
        </TabGroup>
      </div>
      <div className="w-5 h-full" />
    </>
  );
}

function LedgerMenu() {
  const navigate = useNavigate();
  const location = useLocation();
  const params = useParams();

  const reverse = reverseItems();
  const index =
    reverse.length -
    1 -
    reverse.findIndex((item) => location.pathname.endsWith(item.url));
  const menuItems = items();
  return (
    <>
      {!location.pathname.endsWith("/add") ? (
        <TabGroup
          index={index}
          onIndexChange={(index) =>
            navigate(`/ledger/${params.ledgerId}${menuItems[index].url}`)
          }
          className="mb-3 mt-0"
        >
          <TabList className="">
            {menuItems.map((item, index) => (
              <Tab tabIndex={index} key={index}>
                {item.name}
              </Tab>
            ))}
          </TabList>
        </TabGroup>
      ) : (
        <></>
      )}
    </>
  );
}

const items = () => [
  { name: "Transactions", url: "/" },
  { name: "Files", url: "/files" },
  { name: "Meta", url: "/meta" },
];

const reverseItems = () => items().reverse();
