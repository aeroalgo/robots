using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A4 RID: 164
	[HandlerCategory("vvPosClose"), HandlerName("ATR trail Simple v2 speedbeta")]
	public class TrailStopOnATRsimple_v2 : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x060005EE RID: 1518 RVA: 0x0001C980 File Offset: 0x0001AB80
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
			double num;
			if (this._barnumber == barNum && this._atrperiod == this.ATRperiod && this._atr > 0.0)
			{
				num = this._atr;
			}
			else
			{
				num = TrailStop.CalcATR(security, this.ATRperiod, barNum, 0);
				this._atr = num;
				this._barnumber = barNum;
				this._atrperiod = this.ATRperiod;
			}
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

		// Token: 0x17000206 RID: 518
		[HandlerParameter(true, "2", Min = "1", Max = "7", Step = "0.1", Name = "Коэф. ATR")]
		public double ATRcoef
		{
			// Token: 0x060005EA RID: 1514 RVA: 0x0001C95C File Offset: 0x0001AB5C
			get;
			// Token: 0x060005EB RID: 1515 RVA: 0x0001C964 File Offset: 0x0001AB64
			set;
		}

		// Token: 0x17000207 RID: 519
		[HandlerParameter(true, "5", Min = "5", Max = "50", Step = "5", Name = "Период расч. ATR")]
		public int ATRperiod
		{
			// Token: 0x060005EC RID: 1516 RVA: 0x0001C96D File Offset: 0x0001AB6D
			get;
			// Token: 0x060005ED RID: 1517 RVA: 0x0001C975 File Offset: 0x0001AB75
			set;
		}

		// Token: 0x04000211 RID: 529
		private double _atr = -1.0;

		// Token: 0x04000213 RID: 531
		private int _atrperiod = -1;

		// Token: 0x04000212 RID: 530
		private int _barnumber = -1;
	}
}
