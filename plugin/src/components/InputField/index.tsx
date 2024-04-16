import React from 'react'
import './inputField.css'
import ArrayInputField from './ArrayInputField/index'
interface InputFieldProps<T> {
  name?: string
  index: number
  value: T
  type: string
  onChange: (index: number, newValue: T) => void
  placeholder?: string
}

const InputField: React.FC<InputFieldProps<unknown>> = ({
  name,
  index,
  value,
  type,
  onChange,
  placeholder
}) => {
  if (type === 'array') {
    return (
      <ArrayInputField
        name={name}
        index={index}
        type={type}
        value={value as any}
        onChange={onChange}
      />
    )
  }

  return (
    <div className="input-field">
      <label htmlFor={`input-${index}`} className="input-label">
        {name !== '' && name !== undefined ? name + ':' : ''}
      </label>
      <input
        type="text"
        id={`input-${index}`}
        value={value as any}
        onChange={(e) => {
          onChange(index, e.target.value)
        }}
        className="input-text"
        placeholder={placeholder ?? ''}
      />
    </div>
  )
}

export default InputField
