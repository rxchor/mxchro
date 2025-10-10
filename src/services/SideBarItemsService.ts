// Menu items.
import { HomePage } from "@/pages/HomePage";
import { CreateMacroPage } from "@/pages/create_macro/CreateMacroPage";
import { Home, icons } from "lucide-react"
import React from "react";

export interface SideBarItem {
    title: string;
    url: string;
    icon: React.ComponentType<any>;
    component: React.ComponentType<any>;
}

let items: SideBarItem[] = [
  {
    title: "Home",
    url: "/",
    icon: Home,
    component: HomePage,
  },
  {
    title: "Create Macro",
    url: "/create-macro",
    icon: icons.Plus,
    component: CreateMacroPage,
  }
]

export function getSideBarItems(): SideBarItem[] {
    return items;
}