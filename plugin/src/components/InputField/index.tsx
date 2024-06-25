import React from 'react'
import './inputField.css'

interface InputFieldProps {
  name?: string
  index: number
  value: string
  onChange: (index: number, newValue: string) => void
  placeholder?: string
}

const InputField: React.FC<InputFieldProps> = ({ name, index, value, onChange, placeholder }) => {
  return (
    <div className="input-field">
      <label htmlFor={`input-${index}`} className="input-label">
        {name !== '' && name !== undefined ? name + ':' : ''}
      </label>
      <input
        type="text"
        id={`input-${index}`}
        value={value}
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
