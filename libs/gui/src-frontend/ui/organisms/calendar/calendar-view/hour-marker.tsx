type HourMarkerProps = {
  time: string; // e.g. 9 AM
};

export const HourMarker = (props: HourMarkerProps) => (
  <div class="text-text-subtler">
    <div class="flex items-center -translate-y-1/2 w-full">
      <p class="text-[10px] leading-none pt-1 text-right w-12">{props.time}</p>
      <hr class="border-border-subtler w-full ml-2" />
    </div>
  </div>
);
