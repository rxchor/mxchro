import * as React from "react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getSideBarItems } from "@/services/SideBarItemsService";
import { useLocation, useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";

export function AppHeader({ className, ...props }: React.ComponentProps<"header">) {
  const navigate = useNavigate();
  const location = useLocation();
  const items = getSideBarItems();

  return (
    <header
      className={cn(
        "w-full border-b bg-primary-foreground px-4 py-2 flex items-center gap-6",
        className
      )}
      {...props}
    >
      <span className="font-semibold text-lg">mxchro</span>
      <Tabs value={location.pathname} onValueChange={navigate}>
        <TabsList>
          {items.map((item) => (
            <TabsTrigger key={item.title} value={item.url}>
              <item.icon className="size-4" />
              <span>{item.title}</span>
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
    </header>
  );
}
