import { createContext, useCallback, useContext, useState } from 'react';
import { v4 as uuidV4 } from 'uuid';
import type { FlashbarProps } from '@cloudscape-design/components';

type AddFlashbarItemProps = {
  type: FlashbarProps.MessageDefinition['type'];
  header: FlashbarProps.MessageDefinition['header'];
  content: FlashbarProps.MessageDefinition['content'];
};

export interface FlashbarContext {
  /**
   * フラッシュバーのアイテム
   */
  flashbarItems: FlashbarProps.MessageDefinition[];

  /**
   * フラッシュバーのアイテムを追加する関数
   */
  addFlashbarItem: (props: AddFlashbarItemProps) => void;
}

const FlashbarContext = createContext<FlashbarContext>({
  flashbarItems: [],
  addFlashbarItem: () => {
    throw new Error('addFlashbarItems is not implemented');
  },
});

export function useFlashbarContext(): FlashbarContext {
  return useContext(FlashbarContext);
}

export interface FlashbarProviderProps {
  children?: React.ReactNode;
}

export const FlashbarProvider = ({ children }: FlashbarProviderProps): JSX.Element => {
  const [flashbarItems, setFlashbarItems] = useState<FlashbarProps.MessageDefinition[]>([]);

  const addFlashbarItem = useCallback(({ type, header, content }: AddFlashbarItemProps): void => {
    const id = uuidV4();
    setFlashbarItems((prevItems) => [
      ...prevItems,
      {
        type,
        header,
        content,
        dismissible: true,
        dismissLabel: 'close',
        id: id,
        onDismiss: () => {
          setFlashbarItems((items) => items.filter((e) => e.id !== id));
        },
      },
    ]);
  }, []);

  return (
    <FlashbarContext.Provider
      value={{
        flashbarItems,
        addFlashbarItem,
      }}
    >
      {children}
    </FlashbarContext.Provider>
  );
};
