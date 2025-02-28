import React, { useState } from "react";
import axios from "axios";

function App() {
  const [file, setFile] = useState(null);
  const [prompt, setPrompt] = useState("");
  const [response, setResponse] = useState("");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState(""); // Define setMessage


  const handleFileChange = (event) => {
    setFile(event.target.files[0]);
  };

  const handleUpload = async () => {
    if (!file) {
      alert("Please select a file first.");
      return;
    }

    const formData = new FormData();
    formData.append("file", file);

    try {
      const response = await fetch("http://127.0.0.1:8000/upload", {
        method: "POST",
        body: formData,
      });

      if (response.ok) {
        const data = await response.json();
        setMessage(data.message || "File uploaded successfully!");
      } else {
        setMessage("Failed to upload file");
      }
    } catch (error) {
      console.error("Error uploading file:", error);
      setMessage("Error uploading file");
    }
  };

  const handleAsk = async () => {
    if (!prompt) {
      alert("Please enter a question.");
      return;
    }

    setLoading(true);
    try {
      const res = await axios.post("http://127.0.0.1:8000/ask", { prompt });
      setResponse(res.data.response);
    } catch (error) {
      console.error("Error fetching AI response:", error);
      alert("Failed to get response.");
    }
    setLoading(false);
  };

  return (
    <div style={{ maxWidth: "500px", margin: "auto", textAlign: "center", padding: "20px" }}>
      <h2>AI Data Analyst </h2>
      <h3>-developed by anusriesto</h3>
      {/* File Upload */}
      <input type="file" accept=".csv" onChange={handleFileChange} />
      <button onClick={handleUpload}>Upload CSV</button>
      
      {/* Question Input */}
      <div>
        <input
          type="text"
          placeholder="Enter your question"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          style={{ width: "100%", marginTop: "10px", padding: "8px" }}
        />
        <button onClick={handleAsk} disabled={loading}>
          {loading ? "Processing..." : "Ask AI"}
        </button>
      </div>
      
      {/* Response Display */}
      {response && (
        <div style={{ marginTop: "20px", padding: "10px", border: "1px solid #ddd" }}>
          <h4>AI Response:</h4>
          <p>{response}</p>
        </div>
      )}
    </div>
  );
}

export default App;
