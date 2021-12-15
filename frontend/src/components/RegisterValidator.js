import React from "react";

export function RegisterValidator({ registerValidator, address }) {
  return (
    <div>
      <h4>Register validator</h4>
      <form
        onSubmit={(event) => {
          event.preventDefault();

          const formData = new FormData(event.target);
          const nodeUrl = formData.get("nodeUrl");

          if (nodeUrl) {
            registerValidator(nodeUrl);
          }
        }}
      >
        <div className="form-group">
          <label>Your address</label>
          <input
            className="form-control"
            type="text"
            name="validator"
            required
            disabled
            value={address}
          />
        </div>
        <div className="form-group">
          <label>Enter node url:</label>
          <input
            className="form-control"
            type="text"
            name="nodeUrl"
            required
          />
        </div>
        <div className="form-group">
          <input className="btn btn-primary" type="submit" value="Register" />
        </div>
      </form>
    </div>
  );
}
