import {
  Slot,
  component$,
  useContext,
  useComputed$,
  useSignal,
  createContextId,
  useContextProvider,
} from '@builder.io/qwik';
import type { JSXOutput, ContextId, QRL, Signal } from '@builder.io/qwik';

type RouteData<T> = { load: QRL<(data: T) => JSXOutput> };

type RouterContextData<T extends string> = {
  selected: Signal<T>;
  routes: Record<T, RouteData<unknown>>;
};

export function defineRouterContext<T extends string>() {
  return createContextId<RouterContextData<T>>('RouterContext');
}

type StateRouterPortalProps<T extends string> = {
  context: ContextId<RouterContextData<T>>;
};

export const StateRouterPortal = component$(function <T extends string>(
  props: StateRouterPortalProps<T>,
) {
  const context = useContext(props.context);
  const node = useComputed$(() => {
    const routeData = context.routes[context.selected.value];
    // TODO: pass context to router
    return routeData.load(undefined as any);
  });

  return node.value;
});

type StateRouterProviderProps<T extends string> = {
  defaultRoute: T;
  routes: Record<T, RouteData<any>>;
  context: ContextId<RouterContextData<T>>;
};

export const StateRouterProvider = component$(function <T extends string>(
  props: StateRouterProviderProps<T>,
) {
  const context = {
    selected: useSignal(props.defaultRoute),
    routes: props.routes,
  };

  useContextProvider(props.context, context);

  return <Slot />;
});
