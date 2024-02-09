using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A9 RID: 169
	[HandlerCategory("vvPosClose"), HandlerName("TrailStop P1000A-2")]
	public class TrailStopP1000a2 : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600062E RID: 1582 RVA: 0x0001D230 File Offset: 0x0001B430
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFE(barNum);
			double num2 = -this.StopLoss;
			double num3 = pos.get_EntryPrice() + (pos.get_IsLong() ? num2 : (-num2));
			double val = num3;
			if (num > this.TrailEnable)
			{
				if (this.ProfitLimitMax != 0.0 && this.MaxProfitTrLoss != 0.0)
				{
					num2 = ((num > this.ProfitLimitMax) ? (num - this.MaxProfitTrLoss) : (num - this.TrailLoss));
				}
				else if (this.ProfitLimitMin != 0.0 && this.MinProfitTrLoss != 0.0)
				{
					num2 = ((num > this.ProfitLimitMin) ? (num - this.MinProfitTrLoss) : (num - this.TrailLoss));
				}
				else
				{
					num2 = num - this.TrailLoss;
				}
				num3 = pos.get_EntryPrice() + (pos.get_IsLong() ? num2 : (-num2));
			}
			num3 = (pos.get_IsLong() ? Math.Max(num3, val) : Math.Min(num3, val));
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num3;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num3, stop);
			}
			return Math.Max(num3, stop);
		}

		// Token: 0x17000222 RID: 546
		[HandlerParameter(true, "0", Min = "10", Max = "600", Step = "10", Name = "MaxProfitTrLoss")]
		public double MaxProfitTrLoss
		{
			// Token: 0x0600062C RID: 1580 RVA: 0x0001D21C File Offset: 0x0001B41C
			get;
			// Token: 0x0600062D RID: 1581 RVA: 0x0001D224 File Offset: 0x0001B424
			set;
		}

		// Token: 0x17000221 RID: 545
		[HandlerParameter(true, "0", Min = "10", Max = "600", Step = "10", Name = "MinProfitTrLoss")]
		public double MinProfitTrLoss
		{
			// Token: 0x0600062A RID: 1578 RVA: 0x0001D20B File Offset: 0x0001B40B
			get;
			// Token: 0x0600062B RID: 1579 RVA: 0x0001D213 File Offset: 0x0001B413
			set;
		}

		// Token: 0x1700021E RID: 542
		[HandlerParameter(true, "0", Min = "100", Max = "1000", Step = "10", Name = "ProfitLimitMax")]
		public double ProfitLimitMax
		{
			// Token: 0x06000624 RID: 1572 RVA: 0x0001D1D8 File Offset: 0x0001B3D8
			get;
			// Token: 0x06000625 RID: 1573 RVA: 0x0001D1E0 File Offset: 0x0001B3E0
			set;
		}

		// Token: 0x1700021D RID: 541
		[HandlerParameter(true, "0", Min = "100", Max = "1000", Step = "10", Name = "ProfitLimitMin")]
		public double ProfitLimitMin
		{
			// Token: 0x06000622 RID: 1570 RVA: 0x0001D1C7 File Offset: 0x0001B3C7
			get;
			// Token: 0x06000623 RID: 1571 RVA: 0x0001D1CF File Offset: 0x0001B3CF
			set;
		}

		// Token: 0x1700021C RID: 540
		[HandlerParameter(true, "150", Min = "10", Max = "600", Step = "10", Name = "StopLoss")]
		public double StopLoss
		{
			// Token: 0x06000620 RID: 1568 RVA: 0x0001D1B6 File Offset: 0x0001B3B6
			get;
			// Token: 0x06000621 RID: 1569 RVA: 0x0001D1BE File Offset: 0x0001B3BE
			set;
		}

		// Token: 0x1700021F RID: 543
		[HandlerParameter(true, "50", Min = "10", Max = "600", Step = "10", Name = "TrailEnable")]
		public double TrailEnable
		{
			// Token: 0x06000626 RID: 1574 RVA: 0x0001D1E9 File Offset: 0x0001B3E9
			get;
			// Token: 0x06000627 RID: 1575 RVA: 0x0001D1F1 File Offset: 0x0001B3F1
			set;
		}

		// Token: 0x17000220 RID: 544
		[HandlerParameter(true, "150", Min = "10", Max = "600", Step = "10", Name = "TrailLoss")]
		public double TrailLoss
		{
			// Token: 0x06000628 RID: 1576 RVA: 0x0001D1FA File Offset: 0x0001B3FA
			get;
			// Token: 0x06000629 RID: 1577 RVA: 0x0001D202 File Offset: 0x0001B402
			set;
		}
	}
}
