using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A3 RID: 163
	[HandlerCategory("vvPosClose"), HandlerName("ATR trail Simple")]
	public class TrailStopOnATRsimple : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x060005E8 RID: 1512 RVA: 0x0001C890 File Offset: 0x0001AA90
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			pos.OpenMFEPct(barNum);
			IList<double> highPrices = security.get_HighPrices();
			IList<double> lowPrices = security.get_LowPrices();
			IList<double> arg_32_0 = security.get_ClosePrices();
			IList<double> arg_39_0 = security.get_OpenPrices();
			double num = TrailStop.CalcATR(security, this.ATRperiod, barNum, 0);
			double num2 = lowPrices[pos.FindLowBar(barNum)];
			double num3 = highPrices[pos.FindHighBar(barNum)];
			double num4 = num * this.ATRcoef;
			double num5 = pos.get_IsLong() ? (num3 - num4) : (num2 + num4);
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num5 = (pos.get_IsLong() ? Math.Max(num5, stop) : Math.Min(num5, stop));
			}
			return num5;
		}

		// Token: 0x17000204 RID: 516
		[HandlerParameter(true, "2", Min = "1", Max = "7", Step = "0.1", Name = "Коэф. ATR")]
		public double ATRcoef
		{
			// Token: 0x060005E4 RID: 1508 RVA: 0x0001C86E File Offset: 0x0001AA6E
			get;
			// Token: 0x060005E5 RID: 1509 RVA: 0x0001C876 File Offset: 0x0001AA76
			set;
		}

		// Token: 0x17000205 RID: 517
		[HandlerParameter(true, "5", Min = "5", Max = "50", Step = "5", Name = "Период расч. ATR")]
		public int ATRperiod
		{
			// Token: 0x060005E6 RID: 1510 RVA: 0x0001C87F File Offset: 0x0001AA7F
			get;
			// Token: 0x060005E7 RID: 1511 RVA: 0x0001C887 File Offset: 0x0001AA87
			set;
		}
	}
}
