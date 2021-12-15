import React from "react";

export function Bid({ bidForSlot }) {
  return (
    <div>
      <h4>Bid</h4>
      <form
        onSubmit={(event) => {
          // This function just calls the transferTokens callback with the
          // form's data.
          event.preventDefault();

          const formData = new FormData(event.target);
          const amount = formData.get("amount");
          const bidAmount = formData.get("bidAmount");
          const slotNumber = formData.get("slotNumber");

          if (amount && bidAmount && slotNumber) {
            bidForSlot(slotNumber, amount, bidAmount);
          }
        }}
      >
        <div className="form-group">
          <label>Which slot you want to bid?</label>
          <input
            className="form-control"
            type="number"
            step="1"
            name="slotNumber"
            required
          />
        </div>
        <div className="form-group">
          <label>Amount of ether to send</label>
          <input
            className="form-control"
            type="number"
            step="1"
            name="amount"
            placeholder="1"
            required
          />
        </div>
        <div className="form-group">
          <label>How much you want to bid?</label>
          <input
            className="form-control"
            type="number"
            step="1"
            name="bidAmount"
            placeholder="1"
          />
        </div>
        <div className="form-group">
          <input className="btn btn-primary" type="submit" value="Bid" />
        </div>
      </form>
    </div>
  );
}
