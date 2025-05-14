using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000AF RID: 175
	[HandlerCategory("vvPosClose"), HandlerName("Безубыток(%) (+SL)")]
	public class NoLossPctSL : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000667 RID: 1639 RVA: 0x0001D8F4 File Offset: 0x0001BAF4
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFEPct(barNum);
			double result = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (-(this.StopLoss / 100.0)) : (this.StopLoss / 100.0)));
			if (num > this.Profit)
			{
				result = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (this.NoLoss / 100.0) : (-(this.NoLoss / 100.0))));
			}
			return result;
		}

		// Token: 0x17000237 RID: 567
		[HandlerParameter(true, "0.1", Min = "0.05", Max = "0.2", Step = "0.05", Name = "Безубыток")]
		public double NoLoss
		{
			// Token: 0x06000663 RID: 1635 RVA: 0x0001D8CF File Offset: 0x0001BACF
			get;
			// Token: 0x06000664 RID: 1636 RVA: 0x0001D8D7 File Offset: 0x0001BAD7
			set;
		}

		// Token: 0x17000236 RID: 566
		[HandlerParameter(true, "0.3", Min = "0.2", Max = "1", Step = "0.05")]
		public double Profit
		{
			// Token: 0x06000661 RID: 1633 RVA: 0x0001D8BE File Offset: 0x0001BABE
			get;
			// Token: 0x06000662 RID: 1634 RVA: 0x0001D8C6 File Offset: 0x0001BAC6
			set;
		}

		// Token: 0x17000238 RID: 568
		[HandlerParameter(true, "0.2", Min = "0.05", Max = "0.6", Step = "0.05", Name = "Стоп-лосс")]
		public double StopLoss
		{
			// Token: 0x06000665 RID: 1637 RVA: 0x0001D8E0 File Offset: 0x0001BAE0
			get;
			// Token: 0x06000666 RID: 1638 RVA: 0x0001D8E8 File Offset: 0x0001BAE8
			set;
		}
	}
}
