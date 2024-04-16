import React, { useState } from 'react'

interface InputFieldProps<T> {
  name?: string
  index: number
  value: T
  type: string
  onChange: (index: number, newValue: T) => void
  placeholder?: string
}

interface ArrayInputFieldProps extends InputFieldProps<unknown[]> {
  value: unknown[]
}

const ArrayInputField: React.FC<ArrayInputFieldProps> = ({
  name,
  index,
  value,
  onChange
}) => {
  const [items, setItems] = useState(value ?? [])
  const handleAddItem = (): void => {
    const newItem = { value: '' }
    setItems([...items, newItem])
  }

  const handleRemoveItem = (itemIndex: number): void => {
    setItems(items.filter((item, index) => index !== itemIndex))
  }

  const handleClearList: () => void = () => {
    setItems([])
  }

  return (
    <div className="array-input-field">
      <label htmlFor={`input-${index}`} className="input-label">
        {name !== '' && name !== undefined ? name + ':' : ''}
      </label>
      {items.map((item: any, itemIndex) => (
        <div key={itemIndex} className="array-item">
          <input
            type="text"
            value={item.value}
            onChange={(e) => {
              const updatedItem = { ...item, value: e.target.value }
              setItems(
                items.map((item, index) =>
                  index === itemIndex ? updatedItem : item
                )
              )
            }}
          />
          <button
            onClick={() => {
              handleRemoveItem(itemIndex)
            }}
          >
            Remove
          </button>
        </div>
      ))}
      <button onClick={handleAddItem}>Add item</button>
      <button onClick={handleClearList}>Clear list</button>
    </div>
  )
}
export default ArrayInputField
