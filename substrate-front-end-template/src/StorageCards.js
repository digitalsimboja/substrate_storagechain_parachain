import React from 'react'
import {
  Button,
  Card,
  Grid,
  Message,
  Modal,
  Form,
  Label,
} from 'semantic-ui-react'

import { useSubstrateState } from './substrate-lib'
import { TxButton } from './substrate-lib/components'

// --- Change Action ---

const ChangeAction = props => {
  const { storage, setStatus } = props
  const [open, setOpen] = React.useState(false)
  const [formValue, setFormValue] = React.useState({})

  const formChange = key => (ev, el) => {
    setFormValue({ ...formValue, [key]: el.value })
  }

  const confirmAndClose = unsub => {
    setOpen(false)
    if (unsub && typeof unsub === 'function') unsub()
  }

  return (
    <Modal
      onClose={() => setOpen(false)}
      onOpen={() => setOpen(true)}
      open={open}
      trigger={
        <Button basic color="blue">
          Change Action
        </Button>
      }
    >
      <Modal.Header>Change the Stored Action</Modal.Header>
      <Modal.Content>
        <Form>
          <Form.Input
            fluid
            label="Storage ID"
            readOnly
            value={storage.storageId}
          />
          <Form.Input
            fluid
            label="Action"
            placeholder="Choose either of Increment, Decrement, or Idle"
            onChange={formChange('target')}
          />
        </Form>
      </Modal.Content>
      <Modal.Actions>
        <Button basic color="grey" onClick={() => setOpen(false)}>
          Cancel
        </Button>
        <TxButton
          label="Action"
          type="SIGNED-TX"
          setStatus={setStatus}
          onClick={confirmAndClose}
          attrs={{
            palletRpc: 'storagechain',
            callable: 'changeAction',
            inputParams: [storage.storageId, formValue.target],
            paramFields: [true, true],
          }}
        />
      </Modal.Actions>
    </Modal>
  )
}


// --- Buy Kitty ---

const ExecuteAction = props => {
    const { storage, setStatus } = props
    const [open, setOpen] = React.useState(false)
  
    const confirmAndClose = unsub => {
      setOpen(false)
      if (unsub && typeof unsub === 'function') unsub()
    }
  
    return (
      <Modal
        onClose={() => setOpen(false)}
        onOpen={() => setOpen(true)}
        open={open}
        trigger={
          <Button basic color="green">
            Execute Action
          </Button>
        }
      >
        <Modal.Header>Execute Action</Modal.Header>
        <Modal.Content>
          <Form>
            <Form.Input fluid label="Kitty ID" readOnly value={storage.storageId} />
            <Form.Input fluid label="Action" readOnly value={storage.action} />
          </Form>
        </Modal.Content>
        <Modal.Actions>
          <Button basic color="grey" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <TxButton
            label="Buy Kitty"
            type="SIGNED-TX"
            setStatus={setStatus}
            onClick={confirmAndClose}
            attrs={{
              palletRpc: 'storagechain',
              callable: 'execute',
              inputParams: [storage.storageId, storage.action],
              paramFields: [true, true],
            }}
          />
        </Modal.Actions>
      </Modal>
    )
  }
  

const StorageCard = props => {
  const { storage, setStatus } = props
  const { num = null, action = null, storer = null } = storage
  const displayNum = num && num.toJSON()
  const { currentAccount } = useSubstrateState()
  const isSelf = currentAccount.address === storage.storer

  return (
    <Card>
      {isSelf && (
        <Label as="a" floating color="teal">
          Mine
        </Label>
      )}
      <Card.Content>
        <Card.Meta style={{ fontSize: '.9em', overflowWrap: 'break-word' }}>
          Number: {displayNum}
        </Card.Meta>
        <Card.Description>
          <p style={{ overflowWrap: 'break-word' }}>Action: {action}</p>
          <p style={{ overflowWrap: 'break-word' }}>Storer: {storer}</p>
        </Card.Description>
      </Card.Content>
      <Card.Content extra style={{ textAlign: 'center' }}>
        {storer === currentAccount.address ? (
          <>
            <ChangeAction storage={storage} setStatus={setStatus} />
          </>
        ) : (
          <>
            <ExecuteAction storage={storage} setStatus={setStatus} />
          </>
        )}
      </Card.Content>
    </Card>
  )
}

const StorageCards = props => {
  const { storages, setStatus } = props

  if (storages.length === 0) {
    return (
      <Message info>
        <Message.Header>
          No Storage items found... Create one now!&nbsp;
          <span role="img" aria-label="point-down">
            👇
          </span>
        </Message.Header>
      </Message>
    )
  }

  return (
    <Grid columns={3}>
      {storages.map((storage, i) => (
        <Grid.Column key={`storage-${i}`}>
          <StorageCard storage={storage} setStatus={setStatus} />
        </Grid.Column>
      ))}
    </Grid>
  )
}

export default StorageCards
