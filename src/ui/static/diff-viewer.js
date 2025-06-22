class DiffViewer {
    constructor() {
        this.sessionId = window.SESSION_ID;
        this.currentFileIndex = 0;
        this.session = null;
        this.files = [];
        this.appliedChanges = new Set();

        this.init();
    }

    async init() {
        this.showLoading(true);

        try {
            await this.loadSession();
            this.setupEventListeners();
            this.renderInterface();
            this.showLoading(false);
        } catch (error) {
            console.error('Failed to initialize diff viewer:', error);
            this.showError('Failed to load diff session');
        }
    }

    async loadSession() {
        try {
            const response = await fetch(`/api/session/${this.sessionId}`);
            const data = await response.json();

            if (data.error) {
                throw new Error(data.error);
            }

            this.session = data;
            this.files = data.files || [];
            this.appliedChanges = new Set(data.applied_changes || []);

            console.log('Session loaded:', this.session);
        } catch (error) {
            console.error('Error loading session:', error);
            throw error;
        }
    }

    setupEventListeners() {
        // File navigation
        document.getElementById('prev-file').addEventListener('click', () => {
            this.navigateFile(-1);
        });

        document.getElementById('next-file').addEventListener('click', () => {
            this.navigateFile(1);
        });

        // Action buttons
        document.getElementById('apply-all').addEventListener('click', () => {
            this.showConfirmation(
                'Apply All Changes',
                'Are you sure you want to apply all changes in this session?',
                () => this.applyAllChanges()
            );
        });

        document.getElementById('apply-selected').addEventListener('click', () => {
            this.applySelectedChanges();
        });

        document.getElementById('skip-all').addEventListener('click', () => {
            this.showConfirmation(
                'Skip All Changes',
                'Are you sure you want to skip all changes? This will complete the session without applying any changes.',
                () => this.completeSession()
            );
        });

        document.getElementById('complete').addEventListener('click', () => {
            this.showConfirmation(
                'Complete Review',
                'Complete the review session and apply selected changes?',
                () => this.completeSession()
            );
        });

        // Modal handlers
        document.getElementById('modal-confirm').addEventListener('click', () => {
            this.hideModal();
            if (this.pendingAction) {
                this.pendingAction();
                this.pendingAction = null;
            }
        });

        document.getElementById('modal-cancel').addEventListener('click', () => {
            this.hideModal();
            this.pendingAction = null;
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case 'ArrowLeft':
                        e.preventDefault();
                        this.navigateFile(-1);
                        break;
                    case 'ArrowRight':
                        e.preventDefault();
                        this.navigateFile(1);
                        break;
                    case 'Enter':
                        e.preventDefault();
                        this.completeSession();
                        break;
                }
            }
        });
    }

    renderInterface() {
        this.renderFileList();
        this.renderCurrentFile();
        this.updateProgress();
        this.updateStatusBar();
    }

    renderFileList() {
        const fileTabsContainer = document.getElementById('file-tabs');
        fileTabsContainer.innerHTML = '';

        this.files.forEach((file, index) => {
            const tab = document.createElement('div');
            tab.className = `file-tab ${index === this.currentFileIndex ? 'active' : ''}`;
            tab.addEventListener('click', () => {
                this.currentFileIndex = index;
                this.renderInterface();
            });

            const fileName = document.createElement('span');
            fileName.className = 'file-tab-name';
            fileName.textContent = file.file_path;

            const changesCount = document.createElement('span');
            changesCount.className = 'file-tab-changes';
            changesCount.textContent = file.changes.length;

            tab.appendChild(fileName);
            tab.appendChild(changesCount);
            fileTabsContainer.appendChild(tab);
        });
    }

    renderCurrentFile() {
        if (this.files.length === 0) {
            this.showNoFiles();
            return;
        }

        const currentFile = this.files[this.currentFileIndex];

        // Update file info
        document.getElementById('current-file-name').textContent = currentFile.file_path;
        document.getElementById('file-type-badge').textContent = currentFile.file_type;

        // Update content
        const beforeContent = document.getElementById('before-content');
        const afterContent = document.getElementById('after-content');

        beforeContent.textContent = currentFile.original_content;
        afterContent.textContent = currentFile.preview_content;

        // Apply syntax highlighting
        this.applySyntaxHighlighting(beforeContent, currentFile.file_type);
        this.applySyntaxHighlighting(afterContent, currentFile.file_type);

        // Update line counts
        const beforeLines = currentFile.original_content.split('\n').length;
        const afterLines = currentFile.preview_content.split('\n').length;
        document.getElementById('before-line-count').textContent = `${beforeLines} lines`;
        document.getElementById('after-line-count').textContent = `${afterLines} lines`;

        // Render changes
        this.renderChanges(currentFile.changes);

        // Update navigation buttons
        document.getElementById('prev-file').disabled = this.currentFileIndex === 0;
        document.getElementById('next-file').disabled = this.currentFileIndex === this.files.length - 1;
    }

    renderChanges(changes) {
        const changesContainer = document.getElementById('changes-container');
        changesContainer.innerHTML = '';

        if (changes.length === 0) {
            changesContainer.innerHTML = '<p class="no-changes">No changes in this file</p>';
            return;
        }

        changes.forEach(change => {
            const changeElement = this.createChangeElement(change);
            changesContainer.appendChild(changeElement);
        });
    }

    createChangeElement(change) {
        const changeDiv = document.createElement('div');
        changeDiv.className = `change-item ${change.applied ? 'applied' : ''}`;
        changeDiv.dataset.changeId = change.id;

        const header = document.createElement('div');
        header.className = 'change-header';

        const typeSpan = document.createElement('span');
        typeSpan.className = `change-type ${change.change_type}`;
        typeSpan.textContent = this.formatChangeType(change.change_type);

        const lineSpan = document.createElement('span');
        lineSpan.className = 'change-line';
        lineSpan.textContent = `Line ${change.line_number}`;

        header.appendChild(typeSpan);
        header.appendChild(lineSpan);

        const content = document.createElement('div');
        content.className = 'change-content';

        if (change.old_content && change.new_content) {
            content.innerHTML = `
                <div style="color: #dc3545;">- ${this.escapeHtml(change.old_content)}</div>
                <div style="color: #28a745;">+ ${this.escapeHtml(change.new_content)}</div>
            `;
        } else if (change.new_content) {
            content.innerHTML = `<div style="color: #28a745;">+ ${this.escapeHtml(change.new_content)}</div>`;
        } else if (change.old_content) {
            content.innerHTML = `<div style="color: #dc3545;">- ${this.escapeHtml(change.old_content)}</div>`;
        }

        const actions = document.createElement('div');
        actions.className = 'change-actions';

        if (change.applied) {
            const unapplyBtn = document.createElement('button');
            unapplyBtn.className = 'btn btn-warning btn-small';
            unapplyBtn.textContent = '↶ Unapply';
            unapplyBtn.addEventListener('click', () => this.unapplyChange(change.id));
            actions.appendChild(unapplyBtn);
        } else {
            const applyBtn = document.createElement('button');
            applyBtn.className = 'btn btn-success btn-small';
            applyBtn.textContent = '✓ Apply';
            applyBtn.addEventListener('click', () => this.applyChange(change.id));
            actions.appendChild(applyBtn);
        }

        const reasonDiv = document.createElement('div');
        reasonDiv.style.fontSize = '0.8rem';
        reasonDiv.style.color = '#6c757d';
        reasonDiv.style.marginTop = '0.5rem';
        reasonDiv.textContent = `Reason: ${change.reason}`;

        changeDiv.appendChild(header);
        changeDiv.appendChild(content);
        changeDiv.appendChild(reasonDiv);
        changeDiv.appendChild(actions);

        return changeDiv;
    }

    async applyChange(changeId) {
        try {
            const response = await fetch(`/api/session/${this.sessionId}/apply`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ change_id: changeId })
            });

            const result = await response.json();

            if (result.success) {
                this.appliedChanges.add(changeId);
                this.updateChangeElement(changeId, true);
                this.updateProgress();
                console.log('Change applied:', changeId);
            } else {
                this.showError(result.error || 'Failed to apply change');
            }
        } catch (error) {
            console.error('Error applying change:', error);
            this.showError('Failed to apply change');
        }
    }

    async unapplyChange(changeId) {
        try {
            const response = await fetch(`/api/session/${this.sessionId}/unapply`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ change_id: changeId })
            });

            const result = await response.json();

            if (result.success) {
                this.appliedChanges.delete(changeId);
                this.updateChangeElement(changeId, false);
                this.updateProgress();
                console.log('Change unapplied:', changeId);
            } else {
                this.showError(result.error || 'Failed to unapply change');
            }
        } catch (error) {
            console.error('Error unapplying change:', error);
            this.showError('Failed to unapply change');
        }
    }

    updateChangeElement(changeId, applied) {
        const changeElement = document.querySelector(`[data-change-id="${changeId}"]`);
        if (changeElement) {
            if (applied) {
                changeElement.classList.add('applied');
            } else {
                changeElement.classList.remove('applied');
            }

            // Update the action button
            const actions = changeElement.querySelector('.change-actions');
            actions.innerHTML = '';

            if (applied) {
                const unapplyBtn = document.createElement('button');
                unapplyBtn.className = 'btn btn-warning btn-small';
                unapplyBtn.textContent = '↶ Unapply';
                unapplyBtn.addEventListener('click', () => this.unapplyChange(changeId));
                actions.appendChild(unapplyBtn);
            } else {
                const applyBtn = document.createElement('button');
                applyBtn.className = 'btn btn-success btn-small';
                applyBtn.textContent = '✓ Apply';
                applyBtn.addEventListener('click', () => this.applyChange(changeId));
                actions.appendChild(applyBtn);
            }
        }
    }

    async applyAllChanges() {
        const allChanges = this.files.flatMap(file => file.changes);

        for (const change of allChanges) {
            if (!this.appliedChanges.has(change.id)) {
                await this.applyChange(change.id);
            }
        }
    }

    async applySelectedChanges() {
        // Apply all currently applied changes (this is a no-op but shows feedback)
        this.updateProgress();
        this.showSuccess(`${this.appliedChanges.size} changes are currently selected for application`);
    }

    async completeSession() {
        try {
            const response = await fetch(`/api/session/${this.sessionId}/complete`, {
                method: 'POST'
            });

            const result = await response.json();

            if (result.success) {
                this.showSuccess(`Session completed! ${result.applied_changes.length} changes will be applied.`);
                document.getElementById('session-status').textContent = 'Session: Completed';

                // Disable all action buttons
                document.querySelectorAll('.actions-panel .btn, .change-actions .btn').forEach(btn => {
                    btn.disabled = true;
                });

                setTimeout(() => {
                    window.close();
                }, 3000);
            } else {
                this.showError(result.error || 'Failed to complete session');
            }
        } catch (error) {
            console.error('Error completing session:', error);
            this.showError('Failed to complete session');
        }
    }

    navigateFile(direction) {
        const newIndex = this.currentFileIndex + direction;
        if (newIndex >= 0 && newIndex < this.files.length) {
            this.currentFileIndex = newIndex;
            this.renderInterface();
        }
    }

    updateProgress() {
        const totalChanges = this.files.reduce((sum, file) => sum + file.changes.length, 0);
        const appliedCount = this.appliedChanges.size;
        const percentage = totalChanges > 0 ? (appliedCount / totalChanges) * 100 : 0;

        document.getElementById('progress-fill').style.width = `${percentage}%`;
        document.getElementById('progress-text').textContent = `${appliedCount} of ${totalChanges} changes applied`;
    }

    updateStatusBar() {
        if (this.session) {
            document.getElementById('repository-info').textContent = `Repository: ${this.session.repository_name}`;
            document.getElementById('session-status').textContent = `Session: ${this.session.status}`;
        }
    }

    applySyntaxHighlighting(element, fileType) {
        if (window.hljs) {
            element.className = `code-content language-${fileType}`;
            hljs.highlightElement(element);
        }
    }

    formatChangeType(type) {
        const typeMap = {
            'replace': 'MODIFY',
            'insert_after': 'INSERT',
            'insert_before': 'INSERT',
            'delete': 'DELETE',
            'create_file': 'CREATE',
            'delete_file': 'DELETE',
            'replace_range': 'MODIFY',
            'insert_many_after': 'INSERT',
            'insert_many_before': 'INSERT',
            'delete_many': 'DELETE'
        };
        return typeMap[type] || type.toUpperCase();
    }

    showNoFiles() {
        document.getElementById('current-file-name').textContent = 'No files to review';
        document.getElementById('file-type-badge').textContent = '';
        document.getElementById('before-content').textContent = '';
        document.getElementById('after-content').textContent = '';
        document.getElementById('changes-container').innerHTML = '<p class="no-changes">No files to review</p>';
    }

    showLoading(show) {
        const overlay = document.getElementById('loading-overlay');
        overlay.style.display = show ? 'flex' : 'none';
    }

    showConfirmation(title, message, action) {
        document.getElementById('modal-title').textContent = title;
        document.getElementById('modal-message').textContent = message;
        document.getElementById('confirmation-modal').classList.add('show');
        this.pendingAction = action;
    }

    hideModal() {
        document.getElementById('confirmation-modal').classList.remove('show');
    }

    showError(message) {
        console.error(message);
        // You could implement a toast notification here
        alert(`Error: ${message}`);
    }

    showSuccess(message) {
        console.log(message);
        // You could implement a toast notification here
        alert(`Success: ${message}`);
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize the diff viewer when the page loads
document.addEventListener('DOMContentLoaded', () => {
    new DiffViewer();
});

