import {
  CircleStackIcon,
  CurrencyDollarIcon,
  CurrencyEuroIcon,
  CurrencyYenIcon,
} from "@heroicons/react/24/solid";
import SvgNeon from "../assets/neon.svg";
import SvgUbs from "../assets/ubs.svg";
import SvgRevolut from "../assets/revolut.svg";
import SvgIbkr from "../assets/ibkr.svg";
import SvgWise from "../assets/wise.svg";

export const CURRENCIES = [
  {
    name: "Swiss Franc",
    symbol: "CHF",
    icon: CircleStackIcon,
  },
  {
    name: "United States Dollar",
    symbol: "USD",
    icon: CurrencyDollarIcon,
  },
  {
    name: "Euro",
    symbol: "EUR",
    icon: CurrencyEuroIcon,
  },
  {
    name: "Polish złoty",
    symbol: "PLN",
    icon: CircleStackIcon,
  },
  {
    name: "Japanese Yen",
    symbol: "JPY",
    icon: CurrencyYenIcon,
  },
];

export const BANKS = [
  {
    id: "neon",
    name: "Neon",
    icon: SvgNeon,
  },
  {
    id: "revolut",
    name: "Revolut",
    icon: SvgRevolut,
  },
  {
    id: "ibkr",
    name: "Interactive Brokers",
    icon: SvgIbkr,
  },
  {
    id: "wise",
    name: "Wise",
    icon: SvgWise,
  },
  {
    id: "ubs",
    name: "Ubs",
    icon: SvgUbs,
  },
];
