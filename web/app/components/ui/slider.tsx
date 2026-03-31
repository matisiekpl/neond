import * as React from "react"
import { Slider } from "radix-ui"

import { cn } from "~/lib/utils"

function SliderRoot({
  className,
  ...props
}: React.ComponentProps<typeof Slider.Root>) {
  return (
    <Slider.Root
      data-slot="slider"
      className={cn("relative flex w-full touch-none items-center", className)}
      {...props}
    />
  )
}

function SliderTrack({
  className,
  ...props
}: React.ComponentProps<typeof Slider.Track>) {
  return (
    <Slider.Track
      data-slot="slider-track"
      className={cn(
        "relative h-1.5 w-full grow overflow-hidden rounded-full bg-muted",
        className,
      )}
      {...props}
    />
  )
}

function SliderRange({
  className,
  ...props
}: React.ComponentProps<typeof Slider.Range>) {
  return (
    <Slider.Range
      data-slot="slider-range"
      className={cn("absolute h-full bg-primary", className)}
      {...props}
    />
  )
}

function SliderThumb({
  className,
  ...props
}: React.ComponentProps<typeof Slider.Thumb>) {
  return (
    <Slider.Thumb
      data-slot="slider-thumb"
      className={cn(
        "block size-4 rounded-full border border-primary/50 bg-background shadow transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
        className,
      )}
      {...props}
    />
  )
}

export { SliderRoot, SliderTrack, SliderRange, SliderThumb }
