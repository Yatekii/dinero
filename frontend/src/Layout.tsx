import { Card, Tab, TabGroup, TabList } from "@tremor/react";
import {
  Outlet,
  useLocation,
  useNavigate,
  useSearchParams,
} from "react-router-dom";
import { useCookies } from "react-cookie";
import { useEffect, useState } from "react";
import { checkAuthStatus, redirectToAuth, logout } from "./lib/auth";
import { API_URL } from "./main";

const items = () => [
  { name: "Overview", url: "/" },
  { name: "Ledgers", url: "/ledger" },
];

const reverseItems = () => items().reverse();

export function Layout() {
  const [cookies, setCookie] = useCookies(["USERID", "USERNAME"]);
  const [searchParams, setSearchParams] = useSearchParams();
  const [authChecked, setAuthChecked] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  // Handle auth callback with userid/username params
  useEffect(() => {
    if (searchParams.get("userid") && searchParams.get("username")) {
      setCookie("USERID", searchParams.get("userid"));
      setCookie("USERNAME", searchParams.get("username"));
      setSearchParams((prev) => {
        prev.delete("userid");
        prev.delete("username");
        return prev;
      });
    }
  }, [searchParams, setCookie, setSearchParams]);

  // Since loaders handle auth now, just mark as checked immediately
  // and assume we're authenticated if we reach the Layout component
  useEffect(() => {
    setAuthChecked(true);
    setIsAuthenticated(true);
  }, []);

  // Show loading while checking auth
  if (!authChecked) {
    return (
      <Card className="w-screen h-screen flex justify-center items-center">
        <h1 className="text-white text-4xl">Checking authentication...</h1>
      </Card>
    );
  }

  // Show loading while redirecting to auth
  if (!isAuthenticated) {
    return (
      <Card className="w-screen h-screen flex justify-center items-center">
        <h1 className="text-white text-4xl">Redirecting to login...</h1>
      </Card>
    );
  }

  // Show main app when authenticated
  return (
    <Card className="w-screen h-screen flex justify-center items-center">
      <div className="w-screen h-screen overflow-scroll">
        <MainMenu username={cookies.USERNAME || "User"} />
        <Outlet />
      </div>
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
            onClick={() => {
              logout();
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
