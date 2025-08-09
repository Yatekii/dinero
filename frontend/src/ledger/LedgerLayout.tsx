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
import { cx } from "../lib/utils";
import { authenticatedFetch } from "../lib/auth";
import { withAuth } from "../lib/routerAuth";

// eslint-disable-next-line react-refresh/only-export-components
export const ledgersLoader = withAuth(async () => {
  const response = await authenticatedFetch(`${API_URL}/ledgers`);
  
  if (!response.ok) {
    throw new Error(`Failed to fetch ledgers: ${response.status} ${response.statusText}`);
  }
  
  const ledgers = ((await response.json()) as ListLedgerResponse).ledgers;
  ledgers.sort((a, b) => a.name.localeCompare(b.name));

  return { ledgers };
});

export function LedgerLayout() {
  const params = useParams();
  return (
    <div className="mx-full flex flex-row overflow-scroll h-screen">
      <LedgerSelector />
      <div className="w-full ml-48">
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
      <ul className="flex flex-col mb-5 w-48 fixed">
        <li>
          <Button
            variant="primary"
            onClick={() => navigate(`/ledger/add`)}
            className="py-1 px-3 mb-5"
          >
            Add
          </Button>
        </li>
        {ledgers.map((l) => {
          const active = params.ledgerId == l.id;
          return (
            <li
              key={l.id}
              className={cx(
                "text-white hover:bg-slate-700 rounded-md border-slate-400 border-solid border-2 p-2 py-1 mb-2",
                active ? "bg-slate-700" : ""
              )}
            >
              <a href={`/ledger/${l.id}`}>
                <CakeIcon className="h-4 w-4 -mt-2 mr-2 inline" />
                {l.name}
              </a>
            </li>
          );
        })}
      </ul>
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
