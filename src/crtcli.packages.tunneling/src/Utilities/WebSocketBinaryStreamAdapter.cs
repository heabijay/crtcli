using System;
using System.IO;
using System.Net.WebSockets;
using System.Threading;
using System.Threading.Tasks;

namespace CrtCli.Packages.Tunneling.Utilities;

internal class WebSocketBinaryStreamAdapter(WebSocket webSocket) : Stream
{
    private readonly WebSocket _webSocket = webSocket ?? throw new ArgumentNullException(nameof(webSocket));
    
    private bool _isReceiverCompleted;
    

    public override async Task<int> ReadAsync(
        byte[] buffer, 
        int offset, 
        int count,
        CancellationToken cancellationToken)
    {
        if (_isReceiverCompleted)
        {
            return 0;
        }
        
        var result = await _webSocket
            .ReceiveAsync(new ArraySegment<byte>(buffer), cancellationToken)
            .ConfigureAwait(false);

        if (result.MessageType == WebSocketMessageType.Close)
        {
            await _webSocket
                .CloseAsync(WebSocketCloseStatus.NormalClosure, "", cancellationToken)
                .ConfigureAwait(false);
                    
            _isReceiverCompleted = true;
        }

        if (result.MessageType == WebSocketMessageType.Text)
        {
            throw new NotSupportedException("Text messages are not supported");
        }
        
        return result.Count;
    }

    public override async Task WriteAsync(
        byte[] buffer, 
        int offset, 
        int count, 
        CancellationToken cancellationToken)
    {
        await _webSocket.SendAsync(
                new ArraySegment<byte>(buffer, offset, count),
                WebSocketMessageType.Binary,
                true,
                cancellationToken)
            .ConfigureAwait(false);
    }

    public override void Flush()
    {
    }

    public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
    
    public override void SetLength(long value) => throw new NotSupportedException();
    
    public override bool CanRead => _webSocket.State == WebSocketState.Open;
    
    public override bool CanWrite => _webSocket.State == WebSocketState.Open;
    
    public override bool CanSeek => false;
    
    public override long Length => throw new NotSupportedException();

    public override long Position
    {
        get => throw new NotSupportedException();
        set => throw new NotSupportedException();
    }

    public override int Read(byte[] buffer, int offset, int count)
    {
        throw new NotSupportedException();
    }

    public override void Write(byte[] buffer, int offset, int count)
    {
        throw new NotSupportedException();
    }

    public override void Close()
    {
        _webSocket.Abort();
        _webSocket.Dispose();
        
        base.Close();
    }
}