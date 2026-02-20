use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::prepare::{PrepareParams, PrepareResult};
use crate::utils;
use crate::utils::contract;
use ethers::abi::Abi;

#[derive(Clone)]
struct AppState {
    defaults: DefaultParams,
    result: Arc<Mutex<Option<PrepareResult>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultParams {
    contract: Option<String>,
    rpc_url: Option<String>,
    network: Option<String>,
    infura_key: Option<String>,
    from: Option<String>,
    to: Option<String>,
    function_name: Option<String>,
    args: Option<String>,
    value: String,
    output: String,
    gas_limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct FormData {
    contract: String,
    rpc_url: Option<String>,
    network: Option<String>,
    infura_key: Option<String>,
    from: String,
    to: Option<String>,
    function_name: Option<String>,
    args: Option<String>,
    value: String,
    output: String,
    gas_limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AbiRequest {
    contract: String,
    function_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct AbiResponse {
    success: bool,
    params: Vec<ParamInfo>,
    functions: Vec<FunctionInfo>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ParamInfo {
    name: String,
    param_type: String,
}

#[derive(Debug, Serialize)]
struct FunctionInfo {
    name: String,
}

pub async fn execute(
    contract: Option<String>,
    rpc_url: Option<String>,
    network: Option<String>,
    infura_key: Option<String>,
    from: Option<String>,
    to: Option<String>,
    function_name: Option<String>,
    args: Option<String>,
    value: String,
    output: String,
    gas_limit: Option<u64>,
) -> Result<()> {
    let defaults = DefaultParams {
        contract,
        rpc_url,
        network,
        infura_key,
        from,
        to,
        function_name,
        args,
        value,
        output,
        gas_limit,
    };

    // Find available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .context("Failed to bind to localhost")?;
    let addr = listener.local_addr()?;
    drop(listener);

    println!("Starting interactive mode...");
    println!("Server will run on: http://{}", addr);

    let state = AppState {
        defaults,
        result: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/", get(serve_form))
        .route("/prepare", post(handle_prepare))
        .route("/abi", post(handle_abi))
        .with_state(state);

    // Auto-open browser
    let url = format!("http://{}", addr);
    println!("Opening browser at {}", url);
    if let Err(e) = open::that(&url) {
        eprintln!("Failed to open browser: {}. Please open manually: {}", e, url);
    }

    println!("\nWaiting for form submission...\n");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_form(State(state): State<AppState>) -> Html<String> {
    let defaults = &state.defaults;

    let contract_val = defaults.contract.as_deref().unwrap_or("");
    let rpc_url_val = defaults.rpc_url.as_deref().unwrap_or("");
    let network_val = defaults.network.as_deref().unwrap_or("");
    let infura_key_val = defaults.infura_key.as_deref().unwrap_or("");
    let from_val = defaults.from.as_deref().unwrap_or("");
    let to_val = defaults.to.as_deref().unwrap_or("");
    let function_val = defaults.function_name.as_deref().unwrap_or("");
    let args_val = defaults.args.as_deref().unwrap_or("");
    let value_val = &defaults.value;
    let output_val = &defaults.output;
    let gas_limit_val = defaults.gas_limit.map(|g| g.to_string()).unwrap_or_default();

    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>cold-sign prepare (interactive)</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            background: #0d1117;
            color: #c9d1d9;
            font-family: 'Courier New', monospace;
            padding: 20px;
            line-height: 1.6;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
        }}
        h1 {{
            color: #58a6ff;
            margin-bottom: 10px;
            font-size: 24px;
        }}
        .subtitle {{
            color: #8b949e;
            margin-bottom: 30px;
            font-size: 14px;
        }}
        .form-group {{
            margin-bottom: 20px;
        }}
        label {{
            display: block;
            color: #58a6ff;
            margin-bottom: 5px;
            font-size: 14px;
        }}
        .help-text {{
            color: #8b949e;
            font-size: 12px;
            margin-top: 3px;
        }}
        input, select {{
            width: 100%;
            padding: 8px 12px;
            background: #161b22;
            border: 1px solid #30363d;
            color: #c9d1d9;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            border-radius: 6px;
        }}
        input:focus, select:focus {{
            outline: none;
            border-color: #58a6ff;
        }}
        .radio-group {{
            display: flex;
            gap: 20px;
            margin-top: 8px;
        }}
        .radio-option {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        .radio-option input[type="radio"] {{
            width: auto;
        }}
        button {{
            background: #238636;
            color: #ffffff;
            border: none;
            padding: 10px 24px;
            font-size: 14px;
            font-family: 'Courier New', monospace;
            font-weight: bold;
            border-radius: 6px;
            cursor: pointer;
            margin-top: 10px;
        }}
        button:hover {{
            background: #2ea043;
        }}
        button:active {{
            background: #1a7f37;
        }}
        #result {{
            margin-top: 30px;
            padding: 20px;
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 6px;
            display: none;
        }}
        .success {{
            color: #3fb950;
        }}
        .error {{
            color: #f85149;
        }}
        .fieldset {{
            border: 1px solid #30363d;
            padding: 15px;
            margin-bottom: 20px;
            border-radius: 6px;
        }}
        .fieldset legend {{
            color: #58a6ff;
            padding: 0 10px;
            font-weight: bold;
        }}
        .conditional {{
            display: none;
        }}
        .conditional.active {{
            display: block;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 cold-sign prepare</h1>
        <p class="subtitle">Interactive mode - Fill in the parameters below</p>

        <form id="prepareForm">
            <div class="fieldset">
                <legend>Network Configuration</legend>

                <div class="form-group">
                    <label>RPC Method:</label>
                    <div class="radio-group">
                        <div class="radio-option">
                            <input type="radio" id="rpc-direct" name="rpc-method" value="direct" checked>
                            <label for="rpc-direct" style="margin: 0;">Direct RPC URL</label>
                        </div>
                        <div class="radio-option">
                            <input type="radio" id="rpc-infura" name="rpc-method" value="infura">
                            <label for="rpc-infura" style="margin: 0;">Infura Network</label>
                        </div>
                    </div>
                </div>

                <div id="direct-rpc" class="conditional active">
                    <div class="form-group">
                        <label for="rpc_url">RPC URL:</label>
                        <input type="text" id="rpc_url" name="rpc_url" value="{rpc_url_val}" placeholder="https://mainnet.infura.io/v3/YOUR-KEY">
                    </div>
                </div>

                <div id="infura-rpc" class="conditional">
                    <div class="form-group">
                        <label for="network">Network:</label>
                        <select id="network" name="network">
                            <option value="">-- Select Network --</option>
                            <option value="mainnet">Ethereum Mainnet</option>
                            <option value="sepolia">Ethereum Sepolia</option>
                            <option value="holesky">Ethereum Holesky</option>
                            <option value="polygon">Polygon</option>
                            <option value="polygon-amoy">Polygon Amoy</option>
                            <option value="arbitrum">Arbitrum</option>
                            <option value="arbitrum-sepolia">Arbitrum Sepolia</option>
                            <option value="optimism">Optimism</option>
                            <option value="optimism-sepolia">Optimism Sepolia</option>
                            <option value="base">Base</option>
                            <option value="base-sepolia">Base Sepolia</option>
                            <option value="avalanche">Avalanche</option>
                            <option value="avalanche-fuji">Avalanche Fuji</option>
                            <option value="linea">Linea</option>
                            <option value="linea-sepolia">Linea Sepolia</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label for="infura_key">Infura API Key:</label>
                        <input type="text" id="infura_key" name="infura_key" value="{infura_key_val}" placeholder="your-infura-api-key">
                    </div>
                </div>
            </div>

            <div class="fieldset">
                <legend>Contract & Account</legend>

                <div class="form-group">
                    <label for="contract">Contract JSON Path:</label>
                    <input type="text" id="contract" name="contract" value="{contract_val}" placeholder="./Counter.json" required>
                    <p class="help-text">Path to compiled Solidity contract JSON</p>
                </div>

                <div class="form-group">
                    <label for="from">From Address:</label>
                    <input type="text" id="from" name="from" value="{from_val}" placeholder="0x..." required>
                    <p class="help-text">Your wallet address (sender)</p>
                </div>
            </div>

            <div class="fieldset">
                <legend>Transaction Type</legend>

                <div class="form-group">
                    <label>Mode:</label>
                    <div class="radio-group">
                        <div class="radio-option">
                            <input type="radio" id="mode-deploy" name="tx-mode" value="deploy" checked>
                            <label for="mode-deploy" style="margin: 0;">Deploy Contract</label>
                        </div>
                        <div class="radio-option">
                            <input type="radio" id="mode-call" name="tx-mode" value="call">
                            <label for="mode-call" style="margin: 0;">Call Function</label>
                        </div>
                    </div>
                </div>

                <div id="call-fields" class="conditional">
                    <div class="form-group">
                        <label for="to">Contract Address (to):</label>
                        <input type="text" id="to" name="to" value="{to_val}" placeholder="0x...">
                        <p class="help-text">Deployed contract address to call</p>
                    </div>

                    <div class="form-group">
                        <label for="function_name">Function Name:</label>
                        <select id="function_name" name="function_name">
                            <option value="">-- Select Function --</option>
                        </select>
                    </div>
                </div>

                <div id="args-container">
                    <!-- Dynamic parameter fields will be inserted here -->
                </div>

                <div class="form-group" id="args-fallback">
                    <label for="args">Arguments (comma-separated):</label>
                    <input type="text" id="args" name="args" value="{args_val}" placeholder="0x123..., 1000000">
                    <p class="help-text">Constructor args (deploy) or function args (call)</p>
                </div>
            </div>

            <div class="fieldset">
                <legend>Transaction Parameters</legend>

                <div class="form-group">
                    <label for="value">ETH Value (wei):</label>
                    <input type="text" id="value" name="value" value="{value_val}" placeholder="0">
                    <p class="help-text">Amount of ETH to send (in wei)</p>
                </div>

                <div class="form-group">
                    <label for="gas_limit">Gas Limit (optional):</label>
                    <input type="text" id="gas_limit" name="gas_limit" value="{gas_limit_val}" placeholder="3000000">
                    <p class="help-text">Leave empty for default (3,000,000)</p>
                </div>

                <div class="form-group">
                    <label for="output">Output File:</label>
                    <input type="text" id="output" name="output" value="{output_val}" placeholder="unsigned.json">
                </div>
            </div>

            <button type="submit">Prepare Transaction</button>
        </form>

        <div id="result"></div>
    </div>

    <script>
        let currentParams = [];
        let availableFunctions = [];

        // Populate function dropdown with available functions
        function populateFunctionDropdown(functions) {{
            const functionSelect = document.getElementById('function_name');
            const currentValue = functionSelect.value;

            // Clear existing options except the first placeholder
            functionSelect.innerHTML = '<option value="">-- Select Function --</option>';

            // Add function options
            functions.forEach(func => {{
                const option = document.createElement('option');
                option.value = func.name;
                option.textContent = func.name;
                functionSelect.appendChild(option);
            }});

            // Restore previously selected value if it still exists
            if (currentValue && functions.some(f => f.name === currentValue)) {{
                functionSelect.value = currentValue;
            }}

            availableFunctions = functions;
        }}

        // Load available functions from contract ABI
        async function loadAvailableFunctions() {{
            const contractPath = document.getElementById('contract').value;
            if (!contractPath) {{
                document.getElementById('function_name').innerHTML = '<option value="">-- Select Function --</option>';
                availableFunctions = [];
                return;
            }}

            try {{
                const response = await fetch('/abi', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        contract: contractPath,
                        function_name: null
                    }})
                }});

                const result = await response.json();

                if (result.success && result.functions && result.functions.length > 0) {{
                    populateFunctionDropdown(result.functions);
                }} else {{
                    document.getElementById('function_name').innerHTML = '<option value="">-- Select Function --</option>';
                    availableFunctions = [];
                }}
            }} catch (error) {{
                console.error('Failed to load functions:', error);
                document.getElementById('function_name').innerHTML = '<option value="">-- Select Function --</option>';
                availableFunctions = [];
            }}
        }}

        // Fetch and display ABI parameters for constructor or selected function
        async function loadAbiParameters() {{
            console.log('loadAbiParameters called');
            const contractPath = document.getElementById('contract').value;
            console.log('Contract path:', contractPath);

            if (!contractPath) {{
                document.getElementById('args-container').innerHTML = '';
                document.getElementById('args-fallback').style.display = 'none';
                currentParams = [];
                return;
            }}

            const txMode = document.querySelector('input[name="tx-mode"]:checked').value;
            const functionName = txMode === 'call' ? document.getElementById('function_name').value : null;
            console.log('Transaction mode:', txMode);
            console.log('Function name:', functionName);

            // In call mode, don't load parameters until a function is selected
            if (txMode === 'call' && !functionName) {{
                console.log('Call mode but no function selected');
                document.getElementById('args-container').innerHTML = '';
                document.getElementById('args-fallback').style.display = 'none';
                currentParams = [];
                return;
            }}

            try {{
                console.log('Fetching ABI for function:', functionName);
                const response = await fetch('/abi', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        contract: contractPath,
                        function_name: functionName
                    }})
                }});

                const result = await response.json();
                console.log('ABI response:', result);

                if (result.success) {{
                    currentParams = result.params;
                    console.log('Parameters found:', result.params.length, result.params);
                    if (result.params.length > 0) {{
                        console.log('Rendering parameter fields');
                        renderParameterFields(result.params);
                        // Always hide fallback after rendering individual fields
                        // (values have been transferred to individual fields if they were in fallback)
                        document.getElementById('args-fallback').style.display = 'none';
                    }} else {{
                        console.log('No parameters for this function/constructor');
                        // Function/constructor has no parameters - clear the container and hide fallback
                        document.getElementById('args-container').innerHTML = '';
                        document.getElementById('args-fallback').style.display = 'none';
                    }}
                }} else {{
                    console.error('ABI request failed:', result.error);
                    currentParams = [];
                    document.getElementById('args-container').innerHTML = '';
                    // Show fallback only on error as last resort
                    document.getElementById('args-fallback').style.display = 'block';
                }}
            }} catch (error) {{
                console.error('Failed to fetch ABI parameters:', error);
                currentParams = [];
                document.getElementById('args-container').innerHTML = '';
                // Show fallback only on error as last resort
                document.getElementById('args-fallback').style.display = 'block';
            }}
        }}

        function renderParameterFields(params) {{
            console.log('renderParameterFields called with', params.length, 'parameters');
            const container = document.getElementById('args-container');
            console.log('Container element:', container);
            container.innerHTML = '';

            // Check if there are prefilled args from command line
            const fallbackArgs = document.getElementById('args').value;
            const prefilledValues = fallbackArgs ? fallbackArgs.split(',').map(v => v.trim()) : [];

            params.forEach((param, index) => {{
                console.log(`Creating field for param ${{index}}:`, param.name, param.param_type);
                const formGroup = document.createElement('div');
                formGroup.className = 'form-group';

                const label = document.createElement('label');
                label.setAttribute('for', `param-${{index}}`);
                label.textContent = `${{param.name || 'arg' + index}} (${{param.param_type}}):`;

                const input = document.createElement('input');
                input.type = 'text';
                input.id = `param-${{index}}`;
                input.name = `param-${{index}}`;
                input.className = 'param-input';
                input.placeholder = getPlaceholder(param.param_type);
                input.required = true;

                // Pre-fill from command-line args if available
                if (index < prefilledValues.length) {{
                    input.value = prefilledValues[index];
                }}

                formGroup.appendChild(label);
                formGroup.appendChild(input);
                container.appendChild(formGroup);
            }});
            console.log('Finished rendering fields, container children:', container.children.length);
        }}

        function getPlaceholder(paramType) {{
            if (paramType === 'address') return '0x...';
            if (paramType.startsWith('uint') || paramType.startsWith('int')) return '1000000';
            if (paramType === 'bool') return 'true';
            if (paramType === 'string') return 'Enter text';
            if (paramType === 'bytes' || paramType.startsWith('bytes')) return '0x...';
            return '';
        }}

        // Toggle RPC method
        document.querySelectorAll('input[name="rpc-method"]').forEach(radio => {{
            radio.addEventListener('change', (e) => {{
                if (e.target.value === 'direct') {{
                    document.getElementById('direct-rpc').classList.add('active');
                    document.getElementById('infura-rpc').classList.remove('active');
                }} else {{
                    document.getElementById('direct-rpc').classList.remove('active');
                    document.getElementById('infura-rpc').classList.add('active');
                }}
            }});
        }});

        // Toggle transaction mode
        document.querySelectorAll('input[name="tx-mode"]').forEach(radio => {{
            radio.addEventListener('change', async (e) => {{
                if (e.target.value === 'call') {{
                    document.getElementById('call-fields').classList.add('active');
                    // Clear constructor parameters when switching to call mode
                    document.getElementById('args-container').innerHTML = '';
                    // Hide fallback in call mode - individual fields will show when function selected
                    document.getElementById('args-fallback').style.display = 'none';
                    currentParams = [];
                    // Load available functions when switching to call mode
                    await loadAvailableFunctions();
                }} else {{
                    document.getElementById('call-fields').classList.remove('active');
                    // Clear function parameters and load constructor parameters when switching to deploy mode
                    document.getElementById('args-container').innerHTML = '';
                    currentParams = [];
                    await loadAbiParameters();
                }}
            }});
        }});

        // Load functions and parameters when contract path changes
        document.getElementById('contract').addEventListener('change', async () => {{
            const txMode = document.querySelector('input[name="tx-mode"]:checked').value;
            if (txMode === 'call') {{
                await loadAvailableFunctions();
            }} else {{
                await loadAbiParameters();
            }}
        }});

        document.getElementById('contract').addEventListener('blur', async () => {{
            const txMode = document.querySelector('input[name="tx-mode"]:checked').value;
            if (txMode === 'call') {{
                await loadAvailableFunctions();
            }} else {{
                await loadAbiParameters();
            }}
        }});

        // Load function parameters when function is selected
        document.getElementById('function_name').addEventListener('change', loadAbiParameters);

        // Pre-select network if provided
        const networkSelect = document.getElementById('network');
        if ('{network_val}') {{
            networkSelect.value = '{network_val}';
            document.getElementById('rpc-infura').checked = true;
            document.getElementById('direct-rpc').classList.remove('active');
            document.getElementById('infura-rpc').classList.add('active');
        }}

        // Pre-select call mode if to/function provided
        if ('{to_val}' || '{function_val}') {{
            document.getElementById('mode-call').checked = true;
            document.getElementById('call-fields').classList.add('active');
            // Hide fallback when in call mode
            document.getElementById('args-fallback').style.display = 'none';
        }}

        // Initialize on page load
        (async function initializePage() {{
            const txMode = document.querySelector('input[name="tx-mode"]:checked').value;

            // Check if args were provided via command line
            const hasPrefilledArgs = '{args_val}' !== '';

            // Hide fallback initially unless args were prefilled via command line
            if (!hasPrefilledArgs) {{
                document.getElementById('args-fallback').style.display = 'none';
            }}

            if ('{contract_val}') {{
                if (txMode === 'call') {{
                    // Load functions first
                    await loadAvailableFunctions();

                    // Pre-select function if provided
                    if ('{function_val}') {{
                        document.getElementById('function_name').value = '{function_val}';
                        // Load parameters for the selected function
                        await loadAbiParameters();
                    }}
                }} else {{
                    // Load constructor parameters in deploy mode
                    await loadAbiParameters();
                }}
            }}
        }})();

        // Form submission
        document.getElementById('prepareForm').addEventListener('submit', async (e) => {{
            e.preventDefault();

            const formData = new FormData(e.target);
            const rpcMethod = formData.get('rpc-method');
            const txMode = formData.get('tx-mode');

            const data = {{
                contract: formData.get('contract'),
                from: formData.get('from'),
                value: formData.get('value') || '0',
                output: formData.get('output') || 'unsigned.json',
            }};

            // RPC configuration
            if (rpcMethod === 'direct') {{
                data.rpc_url = formData.get('rpc_url');
            }} else {{
                data.network = formData.get('network');
                data.infura_key = formData.get('infura_key');
            }}

            // Transaction mode
            if (txMode === 'call') {{
                data.to = formData.get('to');
                data.function_name = formData.get('function_name');
            }}

            // Optional fields - collect individual parameter values
            if (currentParams.length > 0) {{
                const paramValues = currentParams.map((_, index) => {{
                    const input = document.getElementById(`param-${{index}}`);
                    return input ? input.value.trim() : '';
                }});
                if (paramValues.some(v => v !== '')) {{
                    data.args = paramValues.join(', ');
                }}
            }} else {{
                const args = formData.get('args');
                if (args) data.args = args;
            }}

            const gasLimit = formData.get('gas_limit');
            if (gasLimit) data.gas_limit = parseInt(gasLimit);

            const resultDiv = document.getElementById('result');
            resultDiv.style.display = 'block';
            resultDiv.innerHTML = '<p style="color: #58a6ff;">⏳ Preparing transaction...</p>';

            try {{
                const response = await fetch('/prepare', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(data)
                }});

                const result = await response.json();

                if (response.ok && result.success) {{
                    resultDiv.innerHTML = `
                        <h3 class="success">✓ Success!</h3>
                        <p style="margin-top: 10px; color: #c9d1d9;">${{result.message}}</p>
                        <p style="margin-top: 15px; color: #8b949e;">
                            Transaction details saved to: <strong>${{data.output}}</strong>
                        </p>
                        <p style="margin-top: 10px; color: #8b949e;">
                            You can now close this window and proceed to sign the transaction.
                        </p>
                    `;
                }} else {{
                    resultDiv.innerHTML = `
                        <h3 class="error">✗ Error</h3>
                        <pre style="margin-top: 10px; color: #f85149;">${{result.error || 'Unknown error occurred'}}</pre>
                    `;
                }}

                // Scroll to the result
                resultDiv.scrollIntoView({{ behavior: 'smooth', block: 'end' }});
            }} catch (error) {{
                resultDiv.innerHTML = `
                    <h3 class="error">✗ Error</h3>
                    <pre style="margin-top: 10px; color: #f85149;">${{error.message}}</pre>
                `;
                // Scroll to the result
                resultDiv.scrollIntoView({{ behavior: 'smooth', block: 'end' }});
            }}
        }});
    </script>
</body>
</html>
    "#);

    Html(html)
}

async fn handle_abi(
    Json(req): Json<AbiRequest>,
) -> impl IntoResponse {
    // Parse contract JSON to get ABI
    let result = contract::parse_contract_json(&req.contract);

    match result {
        Ok((_bytecode, abi_value)) => {
            let abi: Result<Abi, _> = serde_json::from_value(abi_value);

            match abi {
                Ok(abi) => {
                    let params = if let Some(func_name) = &req.function_name {
                        // Get function parameters
                        if let Ok(function) = abi.function(func_name) {
                            function.inputs.iter().map(|p| ParamInfo {
                                name: p.name.clone(),
                                param_type: format!("{}", p.kind),
                            }).collect()
                        } else {
                            vec![]
                        }
                    } else {
                        // Get constructor parameters
                        if let Some(constructor) = abi.constructor() {
                            constructor.inputs.iter().map(|p| ParamInfo {
                                name: p.name.clone(),
                                param_type: format!("{}", p.kind),
                            }).collect()
                        } else {
                            vec![]
                        }
                    };

                    // Extract non-pure, non-view functions (state-modifying functions)
                    use ethers::abi::StateMutability;
                    let functions: Vec<FunctionInfo> = abi.functions()
                        .filter(|f| {
                            !matches!(f.state_mutability, StateMutability::Pure | StateMutability::View)
                        })
                        .map(|f| FunctionInfo {
                            name: f.name.clone(),
                        })
                        .collect();

                    (StatusCode::OK, Json(AbiResponse {
                        success: true,
                        params,
                        functions,
                        error: None,
                    }))
                }
                Err(e) => {
                    (StatusCode::BAD_REQUEST, Json(AbiResponse {
                        success: false,
                        params: vec![],
                        functions: vec![],
                        error: Some(format!("Failed to parse ABI: {}", e)),
                    }))
                }
            }
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(AbiResponse {
                success: false,
                params: vec![],
                functions: vec![],
                error: Some(format!("Failed to read contract: {}", e)),
            }))
        }
    }
}

async fn handle_prepare(
    State(state): State<AppState>,
    Json(form_data): Json<FormData>,
) -> impl IntoResponse {
    // Resolve RPC URL
    let rpc_url_result = utils::rpc::resolve_rpc_url(
        form_data.rpc_url,
        form_data.network,
        form_data.infura_key,
    );

    let rpc_url = match rpc_url_result {
        Ok(url) => url,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("RPC configuration error: {}", e)
                }))
            );
        }
    };

    let params = PrepareParams {
        contract: form_data.contract,
        rpc_url,
        from: form_data.from,
        to: form_data.to,
        function_name: form_data.function_name,
        args: form_data.args,
        value: form_data.value,
        output: form_data.output,
        gas_limit: form_data.gas_limit,
    };

    match super::prepare::run(params).await {
        Ok(result) => {
            *state.result.lock().await = Some(result.clone());
            (StatusCode::OK, Json(serde_json::json!(result)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("{:#}", e)
            }))
        ),
    }
}
