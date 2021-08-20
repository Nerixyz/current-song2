type StateClassFn<T> = (state: T) => boolean;
interface TreeObj<T> {
  condClass: { [x: string]: StateClassFn<T> };
  element: HTMLElement;
}
type TreeDescriptor<T> = [HTMLElement, TreeObj<T>['condClass']] | TreeObj<T>;

interface InternalDescriptor<T> {
  element: HTMLElement;
  classes: Array<[string, StateClassFn<T>]>;
}

export function smolTree<T>(...descriptors: Array<TreeDescriptor<T>>): {
  update: (state: T) => void;
} {
  const nodes = descriptors
    .map(descriptor => {
      if (Array.isArray(descriptor)) {
        const classes = Object.entries(descriptor[1]);
        return classes.length === 0
          ? null
          : {
              element: descriptor[0],
              classes,
            };
      } else {
        const classes = Object.entries(descriptor.condClass);
        return classes.length === 0
          ? null
          : {
              element: descriptor.element,
              classes,
            };
      }
    })
    .filter(Boolean) as Array<InternalDescriptor<T>>;

  const condClasses = (state: T, node: InternalDescriptor<T>) => {
    for (const [className, fn] of node.classes) {
      if (fn(state)) {
        node.element.classList.add(className);
      } else {
        node.element.classList.remove(className);
      }
    }
  };
  return {
    update: state => {
      for (const node of nodes) {
        condClasses(state, node);
      }
    },
  };
}
