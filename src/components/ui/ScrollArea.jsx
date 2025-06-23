import React from "react"
import * as ScrollAreaPrimitive from "@radix-ui/react-scroll-area"
import { cn } from "./utils"

const ScrollArea = React.forwardRef(function ScrollArea({
  className,
  children,
  ...props
}, ref) {
  return (
    <ScrollAreaPrimitive.Root
      data-slot="scroll-area"
      className={cn("scroll-area-root", className)}
      ref={ref}
      {...props}
    >
      <ScrollAreaPrimitive.Viewport
        data-slot="scroll-area-viewport"
        className="scroll-area-viewport"
        style={{ overscrollBehavior: 'none' }}
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      <ScrollBar />
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  )
})

function ScrollBar({
  className,
  orientation = "vertical",
  ...props
}) {
  return (
    <ScrollAreaPrimitive.ScrollAreaScrollbar
      data-slot="scroll-area-scrollbar"
      orientation={orientation}
      className={cn(
        "scroll-area-scrollbar",
        orientation === "vertical" && "scroll-area-scrollbar-vertical",
        orientation === "horizontal" && "scroll-area-scrollbar-horizontal",
        className
      )}
      {...props}
    >
      <ScrollAreaPrimitive.ScrollAreaThumb
        data-slot="scroll-area-thumb"
        className="scroll-area-thumb"
      />
    </ScrollAreaPrimitive.ScrollAreaScrollbar>
  )
}

export { ScrollArea, ScrollBar }