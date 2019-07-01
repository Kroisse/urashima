// import * as childProcess from 'child_process';
import * as net from 'net';
// import * as path from 'path';
import * as stream from 'stream';
import { ExtensionContext, OutputChannel, TextDocument, Uri, window, workspace, WorkspaceFolder } from 'vscode';
import {
    DocumentSelector,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    StreamInfo,
} from 'vscode-languageclient';

let defaultClient: LanguageClient | undefined;
const clients = new Map<string, LanguageClient>();

function addSlash(s: string): string {
    return s.endsWith('/') ? s : s + '/';
}

let _sortedWorkspaceFolders: string[] | undefined;
function sortedWorkspaceFolders(): string[] {
    if (_sortedWorkspaceFolders == null) {
        _sortedWorkspaceFolders = workspace.workspaceFolders
            ? workspace.workspaceFolders
                  .map(folder => {
                      return addSlash(folder.uri.toString());
                  })
                  .sort((a, b) => a.length - b.length)
            : [];
    }
    return _sortedWorkspaceFolders;
}
workspace.onDidChangeWorkspaceFolders(() => (_sortedWorkspaceFolders = undefined));

function getOuterMostWorkspaceFolder(folder: WorkspaceFolder): WorkspaceFolder {
    const sorted = sortedWorkspaceFolders();
    for (const element of sorted) {
        let uri = addSlash(folder.uri.toString());
        if (uri.startsWith(element)) {
            const f = workspace.getWorkspaceFolder(Uri.parse(element));
            if (f != null) {
                return f;
            }
        }
    }
    return folder;
}

export function activate(_context: ExtensionContext) {
    let outputChannel: OutputChannel = window.createOutputChannel('Naru');

    function createClient(documentSelector: DocumentSelector, workspaceFolder?: WorkspaceFolder): LanguageClient {
        // let port = 6010 + clients.size;
        // if (defaultClient != null) {
        //     port += 1;
        // }
        const port = 6464;
        let serverOptions: ServerOptions = () => {
            let connection: stream.Duplex | undefined;
            // let proc: childProcess.ChildProcess | undefined = childProcess.spawn(
            //     'cargo',
            //     ['watch', '-x', `run`, `--`, `${port}`],
            //     {
            //         stdio: ['ignore', 'inherit', 'inherit'],
            //         cwd: path.resolve(__dirname, '..', '..', 'urashima-naru-langserver'),
            //     },
            // );
            const cleanup = (_err: boolean) => {
                if (connection != null) {
                    connection.end();
                    connection = undefined;
                }
                // if (!err && proc != null) {
                //     proc.kill();
                //     proc = undefined;
                // }
            };
            // proc.on('error', cleanup);
            // proc.on('exit', cleanup);

            return new Promise<StreamInfo>((resolve, _reject) => {
                const tryConnect = () => {
                    const conn = net
                        .createConnection({ port }, () => {
                            connection = conn;
                            outputChannel.appendLine('Connected!');
                            resolve({ reader: conn, writer: conn, detached: true });
                        })
                        .on('error', err => {
                            outputChannel.appendLine(err.toString());
                            setTimeout(tryConnect, 5000);
                        })
                        .on('close', cleanup);
                };
                tryConnect();
            });
        };
        let clientOptions: LanguageClientOptions = {
            documentSelector,
            diagnosticCollectionName: 'urashima-naru',
            outputChannel,
            workspaceFolder,
        };
        return new LanguageClient('urashima-naru', 'Naru', serverOptions, clientOptions);
    }

    function didOpenTextDocument(document: TextDocument): void {
        // We are only interested in language mode text
        if (document.languageId !== 'naru' || (document.uri.scheme !== 'file' && document.uri.scheme !== 'untitled')) {
            return;
        }

        let uri = document.uri;
        // Untitled files go to a default client.
        if (uri.scheme === 'untitled' && !defaultClient) {
            defaultClient = createClient([{ scheme: 'untitled', language: 'naru' }]);
            defaultClient.start();
            return;
        }
        let folder = workspace.getWorkspaceFolder(uri);
        // Files outside a folder can't be handled. This might depend on the language.
        // Single file languages like JSON might handle files outside the workspace folders.
        if (!folder) {
            return;
        }
        // If we have nested workspace folders we only start a server on the outer most workspace folder.
        folder = getOuterMostWorkspaceFolder(folder);
        const folderUri = folder.uri.toString();
        if (clients.has(folderUri)) {
            return;
        }
        const client = createClient(
            [{ scheme: 'file', language: 'naru', pattern: `${folder.uri.fsPath}/**/*` }],
            folder,
        );
        client.start();
        clients.set(folderUri, client);
    }

    workspace.onDidOpenTextDocument(didOpenTextDocument);
    workspace.textDocuments.forEach(didOpenTextDocument);
    workspace.onDidChangeWorkspaceFolders(event => {
        for (const folder of event.removed) {
            const uri = folder.uri.toString();
            const client = clients.get(uri);
            if (client) {
                clients.delete(uri);
                client.stop();
            }
        }
    });
}

export function deactivate(): Thenable<void> {
    const promises: Thenable<void>[] = [];
    if (defaultClient) {
        promises.push(defaultClient.stop());
    }
    for (let client of clients.values()) {
        promises.push(client.stop());
    }
    return Promise.all(promises).then(() => undefined);
}
