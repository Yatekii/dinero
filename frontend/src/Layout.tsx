import { Card, Tab, TabGroup, TabList } from "@tremor/react";
import {
  Outlet,
  useLocation,
  useNavigate,
  useSearchParams,
} from "react-router-dom";
import { useCookies } from "react-cookie";
import { API_URL } from "./main";

const items = () => [
  { name: "Overview", url: "/" },
  { name: "Ledgers", url: "/ledger" },
];

const reverseItems = () => items().reverse();

export function Layout() {
  const [cookies, setCookie] = useCookies(["USERID", "USERNAME"]);
  const [searchParams, setSearchParams] = useSearchParams();

  if (searchParams.get("userid") && searchParams.get("username")) {
    setCookie("USERID", searchParams.get("userid"));
    setCookie("USERNAME", searchParams.get("username"));
    setSearchParams((prev) => {
      prev.delete("userid");
      prev.delete("username");
      return prev;
    });
  }

  if (!cookies.USERID) {
    document.location.replace(`${API_URL}/auth/oidc`);
  }

  return (
    <Card className="w-screen h-screen flex justify-center items-center">
      {cookies.USERID ? (
        <div className="w-screen h-screen overflow-scroll">
          <MainMenu username={cookies.USERNAME} />
          <Outlet />
        </div>
      ) : (
        <h1 className="text-white text-4xl">Logging in ...</h1>
      )}
    </Card>
  );
}

function MainMenu({ username }: { username: string }) {
  const navigate = useNavigate();
  const location = useLocation();

  const reverse = reverseItems();
  const index =
    reverse.length -
    1 -
    reverse.findIndex((item) => location.pathname.startsWith(item.url));
  const menuItems = items();
  return (
    <>
      <TabGroup
        index={index}
        onIndexChange={(index) => navigate(menuItems[index].url)}
        className="mb-3 flex justify-between"
      >
        <TabList className="mt-4">
          {menuItems.map((item, index) => (
            <Tab tabIndex={index} key={index}>
              {item.name}
            </Tab>
          ))}
        </TabList>

        <div className="flex">
          <Tab className="self-end hover:border-none hover:text-gray-600">
            [{username}]
          </Tab>
          <button
            onClick={async () => {
              await fetch(`${API_URL}/logout`, {
                credentials: "include",
                redirect: "follow",
              });
            }}
            className="p-1 py-1 text-gray-600 mt-5 hover:underline"
          >
            logout
          </button>
        </div>
      </TabGroup>
    </>
  );
}
