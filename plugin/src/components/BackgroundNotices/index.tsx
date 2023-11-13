import React from 'react'

const Notices = [
  'The zkSync Remix Plugin is in Alpha',
  'Solidity contracts are compiled on a server hosted by Nethermind'
]

const BackgroundNotices: React.FC = () => {
  return (
    <div>
      <p className='text-center'>Notices</p>
      {
        <ul className='list-group'>
          {Notices.map((notice, index) => {
            return (
              <li className='list-group-item d-flex justify-content-left align-items-center disabled' key={index}>
                <span className='badge badge-primary badge-pill mr-2'>
                  {index + 1}
                </span>
                {notice}
              </li>
            )
          })}
        </ul>
      }
    </div>
  )
}

export default BackgroundNotices
