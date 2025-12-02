// x402 Integration Test Frontend JavaScript

let currentPaymentRequirements = null;

async function testEndpoint(path) {
    const backendUrl = document.getElementById('backend-url').value.trim();
    const responseDiv = document.getElementById('response');
    
    responseDiv.style.display = 'block';
    responseDiv.className = 'response';
    responseDiv.textContent = 'Loading...';

    try {
        const url = `${backendUrl}${path}`;
        const response = await fetch(url);

        if (response.status === 402) {
            // Payment required
            const paymentReq = await response.json();
            currentPaymentRequirements = paymentReq;
            
            responseDiv.className = 'response error';
            responseDiv.innerHTML = `
                <div style="margin-bottom: 1rem;">
                    <span class="status status-402">402 Payment Required</span>
                </div>
                <strong>Payment Requirements:</strong><br>
                Amount: ${paymentReq.accepts[0]?.maxAmountRequired || 'N/A'} ${paymentReq.accepts[0]?.asset || 'USDC'}<br>
                Network: ${paymentReq.accepts[0]?.network || 'N/A'}<br>
                Description: ${paymentReq.accepts[0]?.description || 'N/A'}<br>
                <br>
                <button onclick="makePayment('${path}')" style="margin-top: 1rem;">Pay and Retry</button>
                <pre style="margin-top: 1rem; background: #f8f9fa; padding: 1rem; border-radius: 4px; overflow-x: auto;">${JSON.stringify(paymentReq, null, 2)}</pre>
            `;
        } else if (response.ok) {
            const data = await response.json();
            const settlementHeader = response.headers.get('X-PAYMENT-RESPONSE');
            
            responseDiv.className = 'response success';
            let html = `
                <div style="margin-bottom: 1rem;">
                    <span class="status status-200">${response.status} OK</span>
                </div>
            `;
            
            if (settlementHeader) {
                html += `<div style="margin-bottom: 1rem; padding: 0.75rem; background: #d4edda; border-radius: 4px;">
                    <strong>Payment Settled!</strong><br>
                    Settlement: ${settlementHeader}
                </div>`;
            }
            
            html += `<pre style="background: #f8f9fa; padding: 1rem; border-radius: 4px; overflow-x: auto;">${JSON.stringify(data, null, 2)}</pre>`;
            responseDiv.innerHTML = html;
        } else {
            const text = await response.text();
            responseDiv.className = 'response error';
            responseDiv.innerHTML = `
                <div style="margin-bottom: 1rem;">
                    <span class="status status-error">${response.status} Error</span>
                </div>
                <pre>${text}</pre>
            `;
        }
    } catch (error) {
        responseDiv.className = 'response error';
        responseDiv.textContent = `Error: ${error.message}`;
    }
}

async function makePayment(path) {
    if (!currentPaymentRequirements || !currentPaymentRequirements.accepts || currentPaymentRequirements.accepts.length === 0) {
        alert('No payment requirements available');
        return;
    }

    const backendUrl = document.getElementById('backend-url').value.trim();
    const privateKey = document.getElementById('private-key').value.trim();
    const payerAddress = document.getElementById('payer-address').value.trim();

    if (!privateKey || !payerAddress) {
        alert('Please provide private key and payer address');
        return;
    }

    const responseDiv = document.getElementById('response');
    responseDiv.className = 'response';
    responseDiv.textContent = 'Creating payment payload...';

    try {
        // In a real implementation, you would use a library like ethers.js or viem
        // For this integration test, we'll make a request to a helper endpoint
        // or use a simple signing library
        
        // For now, we'll show a message that payment creation needs to be implemented
        // In a real scenario, you would:
        // 1. Create EIP-712 typed data from payment requirements
        // 2. Sign with the private key
        // 3. Create PaymentPayload
        // 4. Encode as base64
        // 5. Send request with X-PAYMENT header

        responseDiv.className = 'response error';
        responseDiv.innerHTML = `
            <div style="margin-bottom: 1rem;">
                <strong>Payment Creation</strong>
            </div>
            <p>Payment payload creation requires EIP-712 signing.</p>
            <p>For integration testing, use the Rust client or implement signing in JavaScript.</p>
            <p>See <code>examples/client.rs</code> for reference implementation.</p>
            <br>
            <strong>Payment Requirements:</strong>
            <pre style="background: #f8f9fa; padding: 1rem; border-radius: 4px; margin-top: 1rem;">${JSON.stringify(currentPaymentRequirements, null, 2)}</pre>
        `;
    } catch (error) {
        responseDiv.className = 'response error';
        responseDiv.textContent = `Error creating payment: ${error.message}`;
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    console.log('x402 Integration Test Frontend loaded');
    
    // Check backend health on load
    const backendUrl = document.getElementById('backend-url').value.trim();
    fetch(`${backendUrl}/health`)
        .then(res => res.json())
        .then(data => {
            console.log('Backend health check:', data);
        })
        .catch(err => {
            console.error('Backend health check failed:', err);
        });
});

