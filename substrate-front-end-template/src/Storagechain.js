import React, { useState, useEffect } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { useSubstrateState } from './substrate-lib'
import { TxButton } from './substrate-lib/components'
import StorageCards from './StorageCards'

const parseStorage = ({ num, action, storer }) => ({
  num,
  action: action.toJSON(),
  storer: storer.toJSON(),
})

export default function Storagechain(props) {
  const { api, keyring } = useSubstrateState()
  const [storages, setStorages] = useState([])
  const [storageIds, setStorageIds] = useState([])
  const [status, setStatus] = useState('')
  const [formValue, setFormValue] = React.useState({})

  const formChange = key => (ev, el) => {
    setFormValue({ ...formValue, [key]: el.value })
  }

  // Subscribe to number of storages changes
  const subscribeCount = () => {
    let unsub = null

    const asyncFetch = async () => {
      unsub = await api.query.storagechain.countsOfStorage(async count => {
        // Fetch all the storages keys
        const entries = await api.query.storagechain.storages.entries()
        const ids = entries.map(entry => entry[1].unwrap().num)
        setStorageIds(ids)
      })
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  const subscribeStorages = () => {
    let unsub = null
    const asyncFetch = async () => {
      unsub = await api.query.storagechain.storages.multi(
        storageIds,
        storages => {
          const storagesMap = storages.map(storage =>
            parseStorage(storage.unwrap())
          )
          setStorages(storagesMap)
        }
      )
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  useEffect(subscribeCount, [api, keyring])
  useEffect(subscribeStorages, [api, keyring, storageIds])

  return (
    <Grid.Column width={16}>
      <h1>Storages</h1>
      <StorageCards storages={storages} setStatus={setStatus} />
      <Form style={{ margin: '1em 0' }}>
        <Form.Field style={{ textAlign: 'center' }}>
          <Form.Input
            fluid
            label="Number"
            placeholder="Enter a number"
            onChange={formChange('num')}
          />
        </Form.Field>

        <Form.Field style={{ textAlign: 'center' }}>
          <Form.Input
            fluid
            label="Action"
            placeholder="Choose either of Increment, Decrement, or Idle"
            onChange={formChange('action')}
          />
        </Form.Field>

        <Form.Field style={{ textAlign: 'center' }}>
          <TxButton
            label="Create Storage"
            type="SIGNED-TX"
            setStatus={setStatus}
            attrs={{
              palletRpc: 'storagechain',
              callable: 'store',
              inputParams: [formValue.num, formValue.action],
              paramFields: [true, true],
            }}
          />
        </Form.Field>
      </Form>
      <div style={{ overflowWrap: 'break-word' }}>{status}</div>
    </Grid.Column>
  )
}
