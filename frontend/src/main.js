// Main Application Logic

document.addEventListener('DOMContentLoaded', () => {
  const form = document.getElementById('bfhlForm');
  const edgeInput = document.getElementById('edgeInput');
  const submitBtn = document.getElementById('submitBtn');
  const btnSpinner = document.getElementById('btnSpinner');
  const btnText = document.querySelector('.btn-text');
  
  const errorBox = document.getElementById('errorBox');
  const errorMessage = document.getElementById('errorMessage');
  
  const resultsSection = document.getElementById('resultsSection');
  const userInfoDiv = document.getElementById('userInfo');
  
  const valTotalTrees = document.getElementById('valTotalTrees');
  const valTotalCycles = document.getElementById('valTotalCycles');
  const valLargestRoot = document.getElementById('valLargestRoot');
  
  const issuesContainer = document.getElementById('issuesContainer');
  const invalidSection = document.getElementById('invalidSection');
  const duplicateSection = document.getElementById('duplicateSection');
  const invalidTags = document.getElementById('invalidTags');
  const duplicateTags = document.getElementById('duplicateTags');
  
  const treeView = document.getElementById('treeView');

  // Load API URL from env
  const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/bfhl';

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    // Clear previous results
    hideError();
    resultsSection.style.display = 'none';
    
    // Parse input
    const rawInput = edgeInput.value;
    
    // We allow comma separated or newline separated
    const dataArray = rawInput
      .split(/[,;\n]+/)
      .map(s => s.trim())
      .filter(s => s.length > 0);

    if (dataArray.length === 0) {
      showError("Please enter at least one edge.");
      return;
    }

    const payload = { data: dataArray };

    // Set loading state
    setLoading(true);

    try {
      const response = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`Server returned ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      renderResults(data);

    } catch (err) {
      showError(err.message || "Failed to connect to the server.");
    } finally {
      setLoading(false);
    }
  });

  function setLoading(isLoading) {
    submitBtn.disabled = isLoading;
    if (isLoading) {
      btnText.style.display = 'none';
      btnSpinner.style.display = 'block';
    } else {
      btnText.style.display = 'block';
      btnSpinner.style.display = 'none';
    }
  }

  function showError(msg) {
    errorMessage.textContent = msg;
    errorBox.style.display = 'flex';
  }

  function hideError() {
    errorBox.style.display = 'none';
  }

  function renderResults(data) {
    // 1. User Info
    userInfoDiv.innerHTML = `
      <span>User: <strong>${data.user_id || '-'}</strong></span>
      <span>Email: <strong>${data.email_id || '-'}</strong></span>
      <span>Roll: <strong>${data.college_roll_number || '-'}</strong></span>
    `;

    // 2. Summary Stats
    const { summary } = data;
    valTotalTrees.textContent = summary.total_trees;
    valTotalCycles.textContent = summary.total_cycles;
    valLargestRoot.textContent = summary.largest_tree_root || '-';

    // 3. Issues (Invalid & Duplicates)
    const hasInvalids = data.invalid_entries && data.invalid_entries.length > 0;
    const hasDuplicates = data.duplicate_edges && data.duplicate_edges.length > 0;
    
    if (hasInvalids || hasDuplicates) {
      issuesContainer.style.display = 'flex';
      
      if (hasInvalids) {
        invalidSection.style.display = 'block';
        invalidTags.innerHTML = data.invalid_entries
          .map(item => `<span class="tag">${item}</span>`)
          .join('');
      } else {
        invalidSection.style.display = 'none';
      }

      if (hasDuplicates) {
        duplicateSection.style.display = 'block';
        duplicateTags.innerHTML = data.duplicate_edges
          .map(item => `<span class="tag">${item}</span>`)
          .join('');
      } else {
        duplicateSection.style.display = 'none';
      }
    } else {
      issuesContainer.style.display = 'none';
    }

    // 4. Hierarchies Tree View
    treeView.innerHTML = ''; // Clear old tree
    if (data.hierarchies && data.hierarchies.length > 0) {
      const rootList = document.createElement('ul');
      rootList.className = 'tree-list';

      data.hierarchies.forEach(hierarchy => {
        const item = buildHierarchyElement(hierarchy);
        rootList.appendChild(item);
      });

      treeView.appendChild(rootList);
    } else {
      treeView.innerHTML = '<p style="color: var(--text-secondary)">No valid hierarchies found.</p>';
    }

    // Show results
    resultsSection.style.display = 'flex';
  }

  function buildHierarchyElement(hierarchy) {
    const li = document.createElement('li');
    li.className = 'tree-item';

    // Node container
    const nodeDiv = document.createElement('div');
    nodeDiv.className = 'tree-node';

    // Label
    const labelSpan = document.createElement('span');
    labelSpan.className = 'node-label';
    labelSpan.textContent = hierarchy.root;
    nodeDiv.appendChild(labelSpan);

    // Meta (Depth / Cycle badges)
    const metaDiv = document.createElement('div');
    metaDiv.className = 'node-meta';
    
    if (hierarchy.has_cycle) {
      metaDiv.innerHTML = `<span class="badge cycle">Cycle Detected</span>`;
    } else if (hierarchy.depth !== undefined) {
      metaDiv.innerHTML = `<span class="badge depth">Depth: ${hierarchy.depth}</span>`;
    }
    nodeDiv.appendChild(metaDiv);

    li.appendChild(nodeDiv);

    // Recursively add children if it's a valid tree and has children
    // The backend returns tree as: { "A": { "B": {}, "C": {} } }
    // We need to pass the children of the root.
    if (!hierarchy.has_cycle && hierarchy.tree && hierarchy.tree[hierarchy.root]) {
      const childrenObj = hierarchy.tree[hierarchy.root];
      if (Object.keys(childrenObj).length > 0) {
        const childList = buildTreeNodes(childrenObj);
        li.appendChild(childList);
      }
    }

    return li;
  }

  // Recursive function to build nested UI elements for JSON tree
  function buildTreeNodes(childrenObj) {
    const ul = document.createElement('ul');
    ul.className = 'tree-list';

    for (const [nodeName, nodeChildren] of Object.entries(childrenObj)) {
      const li = document.createElement('li');
      li.className = 'tree-item';

      const nodeDiv = document.createElement('div');
      nodeDiv.className = 'tree-node';
      nodeDiv.innerHTML = `<span class="node-label">${nodeName}</span>`;
      li.appendChild(nodeDiv);

      if (Object.keys(nodeChildren).length > 0) {
        const childList = buildTreeNodes(nodeChildren);
        li.appendChild(childList);
      }

      ul.appendChild(li);
    }

    return ul;
  }
});
