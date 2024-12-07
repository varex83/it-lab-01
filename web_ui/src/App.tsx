import React, { useEffect, useState } from 'react';
import axios from 'axios';
import styled from 'styled-components';
import { Theme } from '@radix-ui/themes';
import '@radix-ui/themes/styles.css';
import * as Select from '@radix-ui/react-select';
import * as Toast from '@radix-ui/react-toast';
import * as Dialog from '@radix-ui/react-dialog';
import * as Tabs from '@radix-ui/react-tabs';
import { CheckIcon, ChevronDownIcon, ChevronUpIcon, Cross2Icon, PlusIcon } from '@radix-ui/react-icons';

interface ConnectionConfig {
  host: string;
  port: string;
}

interface DbValue {
  Integer?: number;
  Real?: number;
  Char?: string;
  String?: string;
  Money?: number;
  MoneyRange?: [number, number];
}

interface DbColumn {
  name: string;
  column_type: string;
}

interface DbSchema {
  columns: DbColumn[];
}

interface DbRecord {
  id: string;
  values: DbValue[];
}

interface TableDetails {
  schema: DbSchema;
  rows: DbRecord[];
}

interface TableList {
  tables: string[];
}

const StyledApp = styled.div`
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
`;

const StyledHeader = styled.header`
  margin-bottom: 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const StyledSection = styled.section`
  margin-bottom: 2rem;
`;

const StyledSelect = styled(Select.Root)`
  position: relative;
  width: 200px;
`;

const StyledTrigger = styled(Select.Trigger)`
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  border-radius: 4px;
  padding: 0 15px;
  font-size: 13px;
  line-height: 1;
  height: 35px;
  gap: 5px;
  background-color: white;
  color: #111;
  border: 1px solid #ccc;
  width: 100%;
  
  &:hover {
    background-color: #f9f9f9;
  }
  
  &:focus {
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.1);
  }
`;

const StyledContent = styled(Select.Content)`
  overflow: hidden;
  background-color: white;
  border-radius: 6px;
  box-shadow: 0px 10px 38px -10px rgba(22, 23, 24, 0.35),
    0px 10px 20px -15px rgba(22, 23, 24, 0.2);
  position: relative;
  z-index: 100;
  min-width: 200px;
`;

const StyledViewport = styled(Select.Viewport)`
  padding: 5px;
`;

const StyledItem = styled(Select.Item)`
  font-size: 13px;
  line-height: 1;
  color: #111;
  border-radius: 3px;
  display: flex;
  align-items: center;
  height: 25px;
  padding: 0 35px 0 25px;
  position: relative;
  user-select: none;

  &[data-highlighted] {
    outline: none;
    background-color: #f5f5f5;
  }

  &[data-state="checked"] {
    background-color: #e9ecef;
  }
`;

const StyledItemIndicator = styled(Select.ItemIndicator)`
  position: absolute;
  left: 0;
  width: 25px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
`;

const SelectWrapper = styled.div`
  position: relative;
  width: 200px;
`;

const StyledToastViewport = styled(Toast.Viewport)`
  position: fixed;
  bottom: 0;
  right: 0;
  display: flex;
  flex-direction: column;
  padding: 25px;
  gap: 10px;
  width: 390px;
  max-width: 100vw;
  margin: 0;
  list-style: none;
  z-index: 2147483647;
  outline: none;
`;

const StyledToast = styled(Toast.Root)`
  background-color: white;
  border-radius: 6px;
  box-shadow: hsl(206 22% 7% / 35%) 0px 10px 38px -10px,
    hsl(206 22% 7% / 20%) 0px 10px 20px -15px;
  padding: 15px;
  display: grid;
  grid-template-areas: 'title action' 'description action';
  grid-template-columns: auto max-content;
  column-gap: 15px;
  align-items: center;
`;

const StyledButton = styled.button<{ variant?: 'primary' | 'secondary' | 'danger' }>`
  border-radius: 4px;
  padding: 0 15px;
  font-size: 13px;
  line-height: 1;
  height: 35px;
  background-color: ${props => 
    props.variant === 'danger' ? '#dc3545' :
    props.variant === 'secondary' ? '#6c757d' :
    '#007bff'};
  color: white;
  border: none;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  
  &:hover {
    background-color: ${props => 
      props.variant === 'danger' ? '#c82333' :
      props.variant === 'secondary' ? '#5a6268' :
      '#0056b3'};
  }
  
  &:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }
`;

const StyledInput = styled.input`
  border-radius: 4px;
  padding: 0 10px;
  font-size: 13px;
  line-height: 1;
  height: 35px;
  border: 1px solid #ccc;
  width: 200px;

  &:focus {
    outline: none;
    border-color: #007bff;
    box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);
  }
`;

const StyledTabs = styled(Tabs.Root)`
  margin-top: 2rem;
`;

const StyledTabsList = styled(Tabs.List)`
  display: flex;
  border-bottom: 1px solid #ccc;
  margin-bottom: 1rem;
`;

const StyledTabsTrigger = styled(Tabs.Trigger)`
  padding: 0.5rem 1rem;
  border: none;
  background: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  color: #666;

  &[data-state="active"] {
    color: #007bff;
    border-bottom-color: #007bff;
  }

  &:hover {
    color: #007bff;
  }
`;

const StyledTabsContent = styled(Tabs.Content)`
  padding: 1rem;
  background: white;
  border-radius: 4px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
`;

const StyledDialog = styled(Dialog.Content)`
  background: white;
  border-radius: 6px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.12);
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 90vw;
  max-width: 450px;
  max-height: 85vh;
  padding: 25px;
`;

const StyledDialogTitle = styled(Dialog.Title)`
  margin: 0 0 1rem;
  font-weight: 500;
  color: #111;
  font-size: 1.2rem;
`;

const StyledDialogClose = styled(Dialog.Close)`
  position: absolute;
  right: 10px;
  top: 10px;
  border: none;
  background: none;
  cursor: pointer;
  padding: 5px;
  color: #666;

  &:hover {
    color: #111;
  }
`;

const StyledForm = styled.form`
  display: flex;
  flex-direction: column;
  gap: 1rem;
`;

const StyledFormField = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
`;

const StyledLabel = styled.label`
  font-size: 13px;
  color: #666;
`;

function App() {
  const [connectionConfig, setConnectionConfig] = useState<ConnectionConfig>({
    host: 'localhost',
    port: '8000'
  });
  const [isConnected, setIsConnected] = useState(false);
  const [tableList, setTableList] = useState<string[]>([]);
  const [selectedTable, setSelectedTable] = useState<string>('');
  const [tableDetails, setTableDetails] = useState<TableDetails | null>(null);
  const [toastOpen, setToastOpen] = useState(false);
  const [toastMessage, setToastMessage] = useState('');
  const [editingRecord, setEditingRecord] = useState<DbRecord | null>(null);
  const [newRecord, setNewRecord] = useState<DbValue[]>([]);
  const [isAddingRecord, setIsAddingRecord] = useState(false);
  const [isCreatingTable, setIsCreatingTable] = useState(false);
  const [newTableName, setNewTableName] = useState('');
  const [newTableColumns, setNewTableColumns] = useState<Array<{ name: string; column_type: string }>>([]);
  const [isCheckingIntersection, setIsCheckingIntersection] = useState(false);
  const [intersectionTable, setIntersectionTable] = useState<string>('');
  const [intersectionResults, setIntersectionResults] = useState<DbRecord[]>([]);

  const showToast = (message: string) => {
    setToastMessage(message);
    setToastOpen(true);
  };

  const handleError = (error: any) => {
    const message = error.response?.data?.message || error.message || 'An error occurred';
    showToast(`Error: ${message}`);
  };

  const checkConnection = async () => {
    try {
      await axios.get(`http://${connectionConfig.host}:${connectionConfig.port}/health`);
      setIsConnected(true);
      showToast('Successfully connected to the server');
    } catch (error) {
      setIsConnected(false);
      handleError(error);
    }
  };

  const fetchTableList = async () => {
    try {
      const response = await axios.get<TableList>(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables`
      );
      setTableList(response.data.tables);
    } catch (error) {
      handleError(error);
      setTableList([]);
    }
  };

  const fetchTableDetails = async (tableName: string) => {
    try {
      const response = await axios.get<TableDetails>(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables/${tableName}/details`
      );
      setTableDetails(response.data);
    } catch (error) {
      handleError(error);
      setTableDetails(null);
    }
  };

  useEffect(() => {
    if (isConnected) {
      fetchTableList();
    }
  }, [isConnected]);

  useEffect(() => {
    if (selectedTable) {
      fetchTableDetails(selectedTable);
    } else {
      setTableDetails(null);
    }
  }, [selectedTable]);

  useEffect(() => {
    if (tableDetails?.schema) {
      setNewRecord(tableDetails.schema.columns.map(() => ({ String: '' })));
    }
  }, [tableDetails?.schema]);

  const formatValue = (value: DbValue): string => {
    if ('Integer' in value && value.Integer !== undefined) return value.Integer.toString();
    if ('Real' in value && value.Real !== undefined) return value.Real.toString();
    if ('Char' in value && value.Char !== undefined) return value.Char;
    if ('String' in value && value.String !== undefined) return value.String;
    if ('Money' in value && value.Money !== undefined) return value.Money.toFixed(2);
    if ('MoneyRange' in value && value.MoneyRange) return `${value.MoneyRange[0]},${value.MoneyRange[1]}`;
    return '';
  };

  const createRecord = async (values: DbValue[]) => {
    try {
      if (!tableDetails || values.length !== tableDetails.schema.columns.length) {
        throw new Error('Invalid number of values');
      }

      const validatedValues = tableDetails.schema.columns.map((column, index) => {
        const value = values[index];
        if (!value || Object.keys(value).length === 0) {
          switch (column.column_type) {
            case 'integer':
              return { Integer: 0 };
            case 'real':
              return { Real: 0.0 };
            case 'char':
              return { Char: '' };
            case 'string':
              return { String: '' };
            case 'money':
              return { Money: 0.0 };
            case 'moneyRange':
              return { MoneyRange: [0.0, 0.0] };
            default:
              return { String: '' };
          }
        }
        return value;
      });

      const response = await axios.post(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables/${selectedTable}/records`,
        { values: validatedValues },
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      );
      
      showToast('Record created successfully');
      fetchTableDetails(selectedTable);
      setNewRecord([]);
      setIsAddingRecord(false);
    } catch (error) {
      handleError(error);
    }
  };

  const updateRecord = async (id: string, values: DbValue[]) => {
    try {
      if (!tableDetails || values.length !== tableDetails.schema.columns.length) {
        throw new Error('Invalid number of values');
      }

      const validatedValues = tableDetails.schema.columns.map((column, index) => {
        const value = values[index];
        if (!value || Object.keys(value).length === 0) {
          switch (column.column_type) {
            case 'integer':
              return { Integer: 0 };
            case 'real':
              return { Real: 0.0 };
            case 'char':
              return { Char: '' };
            case 'string':
              return { String: '' };
            case 'money':
              return { Money: 0.0 };
            case 'moneyRange':
              return { MoneyRange: [0.0, 0.0] };
            default:
              return { String: '' };
          }
        }
        return value;
      });

      const response = await axios.put(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables/${selectedTable}/records/${id}`,
        { values: validatedValues },
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      );
      
      showToast('Record updated successfully');
      fetchTableDetails(selectedTable);
      setEditingRecord(null);
    } catch (error) {
      handleError(error);
    }
  };

  const deleteRecord = async (id: string) => {
    if (!window.confirm('Are you sure you want to delete this record?')) {
      return;
    }

    try {
      await axios.delete(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables/${selectedTable}/records/${id}`
      );
      showToast('Record deleted successfully');
      fetchTableDetails(selectedTable);
    } catch (error) {
      handleError(error);
    }
  };

  const parseValue = (type: string, value: string): DbValue => {
    if (!value) {
      switch (type) {
        case 'integer':
          return { Integer: 0 };
        case 'real':
          return { Real: 0.0 };
        case 'char':
          return { Char: '' };
        case 'string':
          return { String: '' };
        case 'money':
          return { Money: 0.0 };
        case 'moneyRange':
          return { MoneyRange: [0.0, 0.0] };
        default:
          return { String: '' };
      }
    }

    try {
      switch (type) {
        case 'integer':
          return { Integer: parseInt(value) || 0 };
        case 'real':
          return { Real: parseFloat(value) || 0.0 };
        case 'char':
          return { Char: value.charAt(0) || '' };
        case 'string':
          return { String: value };
        case 'money':
          return { Money: parseFloat(value) || 0.0 };
        case 'moneyRange':
          const [min, max] = value.split(',').map(v => parseFloat(v.trim()) || 0.0);
          return { MoneyRange: [min, max] };
        default:
          return { String: value };
      }
    } catch (error) {
      console.error('Value parsing error:', error);
      switch (type) {
        case 'integer':
          return { Integer: 0 };
        case 'real':
          return { Real: 0.0 };
        case 'char':
          return { Char: '' };
        case 'money':
          return { Money: 0.0 };
        case 'moneyRange':
          return { MoneyRange: [0.0, 0.0] };
        default:
          return { String: '' };
      }
    }
  };

  const createTable = async () => {
    try {
      if (!newTableName.trim()) {
        showToast('Table name is required');
        return;
      }
      if (newTableColumns.length === 0) {
        showToast('At least one column is required');
        return;
      }

      await axios.post(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables/${newTableName}`,
        { columns: newTableColumns },
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      );
      
      showToast('Table created successfully');
      setIsCreatingTable(false);
      setNewTableName('');
      setNewTableColumns([]);
      fetchTableList();
    } catch (error) {
      handleError(error);
    }
  };

  const deleteTable = async (tableName: string) => {
    if (!window.confirm(`Are you sure you want to delete table "${tableName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      await axios.delete(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/tables/${tableName}`
      );
      
      showToast('Table deleted successfully');
      setSelectedTable('');
      fetchTableList();
    } catch (error) {
      handleError(error);
    }
  };

  const addColumn = () => {
    setNewTableColumns([...newTableColumns, { name: '', column_type: 'string' }]);
  };

  const removeColumn = (index: number) => {
    setNewTableColumns(newTableColumns.filter((_, i) => i !== index));
  };

  const checkIntersection = async () => {
    if (!selectedTable || !intersectionTable) {
      showToast('Please select both tables for intersection');
      return;
    }

    try {
      const response = await axios.get<DbRecord[]>(
        `http://${connectionConfig.host}:${connectionConfig.port}/api/intersection/${selectedTable}/${intersectionTable}`
      );
      
      setIntersectionResults(response.data);
    } catch (error) {
      handleError(error);
      setIntersectionResults([]);
    }
  };

  return (
    <Theme>
      <Toast.Provider swipeDirection="right">
        <StyledApp>
          <StyledHeader>
            <h1>Database Interface</h1>
            <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: isConnected ? '#28a745' : '#dc3545' }} />
                <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
              </div>
            </div>
          </StyledHeader>

          <StyledSection>
            <h2>Connection Settings</h2>
            <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', marginBottom: '1rem' }}>
              <StyledInput
                type="text"
                value={connectionConfig.host}
                onChange={(e) => setConnectionConfig(prev => ({ ...prev, host: e.target.value }))}
                placeholder="Host"
              />
              <StyledInput
                type="text"
                value={connectionConfig.port}
                onChange={(e) => setConnectionConfig(prev => ({ ...prev, port: e.target.value }))}
                placeholder="Port"
              />
              <StyledButton onClick={checkConnection}>
                {isConnected ? 'Reconnect' : 'Connect'}
              </StyledButton>
            </div>
          </StyledSection>

          {isConnected && (
            <StyledSection>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <h2>Tables</h2>
                <div style={{ display: 'flex', gap: '0.5rem' }}>
                  {selectedTable && (
                    <StyledButton variant="danger" onClick={() => deleteTable(selectedTable)}>
                      <Cross2Icon /> Delete Table
                    </StyledButton>
                  )}
                  <StyledButton onClick={() => setIsCreatingTable(true)}>
                    <PlusIcon /> New Table
                  </StyledButton>
                </div>
              </div>

              <SelectWrapper>
                <StyledSelect value={selectedTable} onValueChange={setSelectedTable}>
                  <StyledTrigger aria-label="Select table">
                    <Select.Value placeholder="Select a table..." />
                    <Select.Icon>
                      <ChevronDownIcon />
                    </Select.Icon>
                  </StyledTrigger>
                  <StyledContent>
                    <Select.ScrollUpButton>
                      <ChevronUpIcon />
                    </Select.ScrollUpButton>
                    <StyledViewport>
                      {tableList.map((table) => (
                        <StyledItem key={table} value={table}>
                          <StyledItemIndicator>
                            <CheckIcon />
                          </StyledItemIndicator>
                          <Select.ItemText>{table}</Select.ItemText>
                        </StyledItem>
                      ))}
                    </StyledViewport>
                    <Select.ScrollDownButton>
                      <ChevronDownIcon />
                    </Select.ScrollDownButton>
                  </StyledContent>
                </StyledSelect>
              </SelectWrapper>

              {/* Create Table Dialog */}
              <Dialog.Root open={isCreatingTable} onOpenChange={setIsCreatingTable}>
                <StyledDialog>
                  <StyledDialogTitle>Create New Table</StyledDialogTitle>
                  <StyledDialogClose asChild>
                    <button>
                      <Cross2Icon />
                    </button>
                  </StyledDialogClose>
                  
                  <StyledForm onSubmit={(e) => { e.preventDefault(); createTable(); }}>
                    <StyledFormField>
                      <StyledLabel>Table Name</StyledLabel>
                      <StyledInput
                        type="text"
                        value={newTableName}
                        onChange={(e) => setNewTableName(e.target.value)}
                        placeholder="Enter table name"
                        required
                      />
                    </StyledFormField>

                    <div style={{ marginTop: '1rem' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
                        <h4>Columns</h4>
                        <StyledButton type="button" onClick={addColumn}>
                          <PlusIcon /> Add Column
                        </StyledButton>
                      </div>

                      {newTableColumns.map((column, index) => (
                        <div key={index} style={{ display: 'flex', gap: '1rem', marginBottom: '0.5rem' }}>
                          <StyledInput
                            type="text"
                            value={column.name}
                            onChange={(e) => {
                              const updated = [...newTableColumns];
                              updated[index].name = e.target.value;
                              setNewTableColumns(updated);
                            }}
                            placeholder="Column name"
                            required
                          />
                          <SelectWrapper>
                            <StyledSelect
                              value={column.column_type}
                              onValueChange={(value) => {
                                const updated = [...newTableColumns];
                                updated[index].column_type = value;
                                setNewTableColumns(updated);
                              }}
                            >
                              <StyledTrigger>
                                <Select.Value />
                              </StyledTrigger>
                              <StyledContent>
                                <StyledViewport>
                                  {['integer', 'real', 'char', 'string', 'money', 'moneyRange'].map((type) => (
                                    <StyledItem key={type} value={type}>
                                      <Select.ItemText>{type}</Select.ItemText>
                                    </StyledItem>
                                  ))}
                                </StyledViewport>
                              </StyledContent>
                            </StyledSelect>
                          </SelectWrapper>
                          <StyledButton
                            type="button"
                            variant="danger"
                            onClick={() => removeColumn(index)}
                          >
                            <Cross2Icon />
                          </StyledButton>
                        </div>
                      ))}
                    </div>

                    <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '1rem' }}>
                      <StyledButton
                        type="button"
                        variant="secondary"
                        onClick={() => {
                          setIsCreatingTable(false);
                          setNewTableName('');
                          setNewTableColumns([]);
                        }}
                      >
                        Cancel
                      </StyledButton>
                      <StyledButton type="submit">
                        Create Table
                      </StyledButton>
                    </div>
                  </StyledForm>
                </StyledDialog>
              </Dialog.Root>

              {selectedTable && tableDetails && (
                <div style={{ marginTop: '2rem' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                    <h3>Records - {selectedTable}</h3>
                    <div style={{ display: 'flex', gap: '0.5rem' }}>
                      <StyledButton onClick={() => setIsAddingRecord(true)}>
                        <PlusIcon /> Add Record
                      </StyledButton>
                      <StyledButton onClick={() => setIsCheckingIntersection(true)}>
                        Find Intersection
                      </StyledButton>
                    </div>
                  </div>

                  {isAddingRecord && (
                    <div style={{ 
                      marginBottom: '1rem', 
                      padding: '1rem', 
                      border: '1px solid #eee', 
                      borderRadius: '4px',
                      background: '#f9f9f9'
                    }}>
                      <h4 style={{ marginBottom: '1rem' }}>New Record</h4>
                      <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
                        {tableDetails.schema.columns.map((column, index) => (
                          <StyledFormField key={index}>
                            <StyledLabel>{column.name} ({column.column_type})</StyledLabel>
                            <StyledInput
                              type={column.column_type === 'integer' || column.column_type === 'real' || column.column_type === 'money' ? 'number' : 'text'}
                              value={newRecord[index] ? formatValue(newRecord[index]) : ''}
                              onChange={(e) => {
                                const newValues = [...newRecord];
                                newValues[index] = parseValue(column.column_type, e.target.value);
                                setNewRecord(newValues);
                              }}
                              placeholder={`Enter ${column.column_type}`}
                            />
                          </StyledFormField>
                        ))}
                      </div>
                      <div style={{ display: 'flex', gap: '0.5rem', marginTop: '1rem' }}>
                        <StyledButton onClick={() => createRecord(newRecord)}>
                          Save
                        </StyledButton>
                        <StyledButton variant="secondary" onClick={() => {
                          setIsAddingRecord(false);
                          setNewRecord(tableDetails.schema.columns.map(() => ({ String: '' })));
                        }}>
                          Cancel
                        </StyledButton>
                      </div>
                    </div>
                  )}

                  <div style={{ overflowX: 'auto', marginTop: '1rem' }}>
                    <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                      <thead>
                        <tr>
                          <th style={{ padding: '0.75rem', textAlign: 'left', borderBottom: '2px solid #eee', fontWeight: 600 }}>ID</th>
                          {tableDetails.schema.columns.map((column, index) => (
                            <th key={index} style={{ padding: '0.75rem', textAlign: 'left', borderBottom: '2px solid #eee', fontWeight: 600 }}>
                              {column.name}
                              <div style={{ fontSize: '0.8em', color: '#666', fontWeight: 'normal' }}>
                                ({column.column_type})
                              </div>
                            </th>
                          ))}
                          <th style={{ padding: '0.75rem', textAlign: 'right', borderBottom: '2px solid #eee', fontWeight: 600 }}>Actions</th>
                        </tr>
                      </thead>
                      <tbody>
                        {tableDetails.rows.map((row) => (
                          <tr key={row.id}>
                            <td style={{ padding: '0.75rem', borderBottom: '1px solid #eee' }}>{row.id}</td>
                            {row.values.map((value, index) => (
                              <td key={index} style={{ padding: '0.75rem', borderBottom: '1px solid #eee' }}>
                                {editingRecord?.id === row.id ? (
                                  <StyledInput
                                    type={tableDetails.schema.columns[index].column_type === 'integer' || 
                                          tableDetails.schema.columns[index].column_type === 'real' || 
                                          tableDetails.schema.columns[index].column_type === 'money' ? 'number' : 'text'}
                                    value={editingRecord.values[index] ? formatValue(editingRecord.values[index]) : ''}
                                    onChange={(e) => {
                                      const newValues = [...editingRecord.values];
                                      try {
                                        newValues[index] = parseValue(tableDetails.schema.columns[index].column_type, e.target.value);
                                        setEditingRecord({ ...editingRecord, values: newValues });
                                      } catch (error) {
                                        console.error('Value parsing error:', error);
                                      }
                                    }}
                                    placeholder={`Enter ${tableDetails.schema.columns[index].column_type}`}
                                  />
                                ) : (
                                  formatValue(value)
                                )}
                              </td>
                            ))}
                            <td style={{ padding: '0.75rem', borderBottom: '1px solid #eee', textAlign: 'right' }}>
                              {editingRecord?.id === row.id ? (
                                <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
                                  <StyledButton onClick={() => updateRecord(row.id, editingRecord.values)}>
                                    Save
                                  </StyledButton>
                                  <StyledButton variant="secondary" onClick={() => setEditingRecord(null)}>
                                    Cancel
                                  </StyledButton>
                                </div>
                              ) : (
                                <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
                                  <StyledButton variant="secondary" onClick={() => setEditingRecord(row)}>
                                    Edit
                                  </StyledButton>
                                  <StyledButton variant="danger" onClick={() => deleteRecord(row.id)}>
                                    Delete
                                  </StyledButton>
                                </div>
                              )}
                            </td>
                          </tr>
                        ))}
                        {tableDetails.rows.length === 0 && (
                          <tr>
                            <td 
                              colSpan={tableDetails.schema.columns.length + 2} 
                              style={{ 
                                padding: '2rem', 
                                textAlign: 'center', 
                                color: '#666',
                                borderBottom: '1px solid #eee'
                              }}
                            >
                              No records found
                            </td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {/* Intersection Dialog */}
              <Dialog.Root open={isCheckingIntersection} onOpenChange={setIsCheckingIntersection}>
                <StyledDialog>
                  <StyledDialogTitle>Find Intersection</StyledDialogTitle>
                  <StyledDialogClose asChild>
                    <button>
                      <Cross2Icon />
                    </button>
                  </StyledDialogClose>
                  
                  <div style={{ marginBottom: '1rem' }}>
                    <p style={{ fontSize: '0.9rem', color: '#666' }}>
                      Find records that exist in both tables with matching values.
                    </p>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <StyledFormField>
                      <StyledLabel>First Table</StyledLabel>
                      <div style={{ color: '#666' }}>{selectedTable}</div>
                    </StyledFormField>

                    <StyledFormField>
                      <StyledLabel>Second Table</StyledLabel>
                      <SelectWrapper>
                        <StyledSelect value={intersectionTable} onValueChange={setIntersectionTable}>
                          <StyledTrigger>
                            <Select.Value placeholder="Select a table..." />
                          </StyledTrigger>
                          <StyledContent>
                            <StyledViewport>
                              {tableList
                                .filter(table => table !== selectedTable)
                                .map((table) => (
                                  <StyledItem key={table} value={table}>
                                    <Select.ItemText>{table}</Select.ItemText>
                                  </StyledItem>
                                ))}
                            </StyledViewport>
                          </StyledContent>
                        </StyledSelect>
                      </SelectWrapper>
                    </StyledFormField>

                    {intersectionResults.length > 0 && (
                      <div style={{ marginTop: '1rem' }}>
                        <h4 style={{ marginBottom: '0.5rem' }}>Intersection Results ({intersectionResults.length})</h4>
                        <div style={{ overflowX: 'auto' }}>
                          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                            <thead>
                              <tr>
                                <th style={{ padding: '0.75rem', textAlign: 'left', borderBottom: '2px solid #eee', fontWeight: 600 }}>ID</th>
                                {tableDetails.schema.columns.map((column, index) => (
                                  <th key={index} style={{ padding: '0.75rem', textAlign: 'left', borderBottom: '2px solid #eee', fontWeight: 600 }}>
                                    {column.name}
                                    <div style={{ fontSize: '0.8em', color: '#666', fontWeight: 'normal' }}>
                                      ({column.column_type})
                                    </div>
                                  </th>
                                ))}
                              </tr>
                            </thead>
                            <tbody>
                              {intersectionResults.map((record) => (
                                <tr key={record.id}>
                                  <td style={{ padding: '0.75rem', borderBottom: '1px solid #eee' }}>{record.id}</td>
                                  {record.values.map((value, index) => (
                                    <td key={index} style={{ padding: '0.75rem', borderBottom: '1px solid #eee' }}>
                                      {formatValue(value)}
                                    </td>
                                  ))}
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>
                      </div>
                    )}
                  </div>

                  <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '1rem' }}>
                    <StyledButton
                      variant="secondary"
                      onClick={() => {
                        setIsCheckingIntersection(false);
                        setIntersectionTable('');
                        setIntersectionResults([]);
                      }}
                    >
                      Close
                    </StyledButton>
                    <StyledButton onClick={checkIntersection} disabled={!intersectionTable}>
                      Find Intersection
                    </StyledButton>
                  </div>
                </StyledDialog>
              </Dialog.Root>
            </StyledSection>
          )}

          <Toast.Provider>
            <StyledToastViewport />
            <StyledToast open={toastOpen} onOpenChange={setToastOpen}>
              <Toast.Title>{toastMessage}</Toast.Title>
              <Toast.Action asChild altText="Close">
                <button onClick={() => setToastOpen(false)}>Close</button>
              </Toast.Action>
            </StyledToast>
          </Toast.Provider>
        </StyledApp>
      </Toast.Provider>
    </Theme>
  );
}

export default App; 