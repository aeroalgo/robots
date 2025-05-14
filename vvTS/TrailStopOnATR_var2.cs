using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A2 RID: 162
	[HandlerCategory("vvPosClose"), HandlerName("ATR trail var2")]
	public class TrailStopOnATR_var2 : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs, IContextUses
	{
		// Token: 0x060005E0 RID: 1504 RVA: 0x0001C6D4 File Offset: 0x0001A8D4
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			double num = pos.OpenMFEPct(barNum);
			double num2 = (0.0 - this.StopLoss) / 100.0;
			double num3 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num2 : (-num2)));
			if (num > this.TrailEnable)
			{
				IList<double> highPrices = security.get_HighPrices();
				IList<double> lowPrices = security.get_LowPrices();
				IList<double> arg_7A_0 = security.get_ClosePrices();
				IList<double> arg_81_0 = security.get_OpenPrices();
				double num4;
				if (this._barnumber == barNum && this._atrperiod == this.ATRperiod && this._atr > 0.0)
				{
					num4 = this._atr;
				}
				else
				{
					num4 = TrailStop.CalcATR(security, this.ATRperiod, barNum, 0);
					this._atr = num4;
					this._barnumber = barNum;
					this._atrperiod = this.ATRperiod;
				}
				double num5 = lowPrices[pos.FindLowBar(barNum)];
				double num6 = highPrices[pos.FindHighBar(barNum)];
				num2 = num4 * this.ATRcoef;
				num3 = (pos.get_IsLong() ? (num6 - num2) : (num5 + num2));
			}
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num3 = (pos.get_IsLong() ? Math.Max(num3, stop) : Math.Min(num3, stop));
			}
			return num3;
		}

		// Token: 0x17000201 RID: 513
		[HandlerParameter(true, "2", Min = "1", Max = "10", Step = "1", Name = "Коэф. ATR")]
		public double ATRcoef
		{
			// Token: 0x060005DC RID: 1500 RVA: 0x0001C6B1 File Offset: 0x0001A8B1
			get;
			// Token: 0x060005DD RID: 1501 RVA: 0x0001C6B9 File Offset: 0x0001A8B9
			set;
		}

		// Token: 0x17000202 RID: 514
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1", Name = "Период расч. ATR")]
		public int ATRperiod
		{
			// Token: 0x060005DE RID: 1502 RVA: 0x0001C6C2 File Offset: 0x0001A8C2
			get;
			// Token: 0x060005DF RID: 1503 RVA: 0x0001C6CA File Offset: 0x0001A8CA
			set;
		}

		// Token: 0x17000203 RID: 515
		public IContext Context
		{
			// Token: 0x060005E1 RID: 1505 RVA: 0x0001C838 File Offset: 0x0001AA38
			get;
			// Token: 0x060005E2 RID: 1506 RVA: 0x0001C840 File Offset: 0x0001AA40
			set;
		}

		// Token: 0x170001FF RID: 511
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Stop Loss (в %)")]
		public double StopLoss
		{
			// Token: 0x060005D8 RID: 1496 RVA: 0x0001C68F File Offset: 0x0001A88F
			get;
			// Token: 0x060005D9 RID: 1497 RVA: 0x0001C697 File Offset: 0x0001A897
			set;
		}

		// Token: 0x17000200 RID: 512
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Trail Enable (в %)")]
		public double TrailEnable
		{
			// Token: 0x060005DA RID: 1498 RVA: 0x0001C6A0 File Offset: 0x0001A8A0
			get;
			// Token: 0x060005DB RID: 1499 RVA: 0x0001C6A8 File Offset: 0x0001A8A8
			set;
		}

		// Token: 0x04000207 RID: 519
		private double _atr = -1.0;

		// Token: 0x04000209 RID: 521
		private int _atrperiod = -1;

		// Token: 0x04000208 RID: 520
		private int _barnumber = -1;
	}
}
