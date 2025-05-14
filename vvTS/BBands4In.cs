using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200011B RID: 283
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("BBands 4inp"), InputInfo(0, "Цена"), InputInfo(2, "Периоды"), InputInfo(3, "Отклонения"), InputInfo(1, "Средняя"), InputsCount(4)]
	public class BBands4In : IFourSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060007FF RID: 2047 RVA: 0x00022410 File Offset: 0x00020610
		public IList<double> Execute(IList<double> src, IList<double> Ma, IList<double> Periods, IList<double> stdevs)
		{
			return this.Context.GetData("bb4in", new string[]
			{
				Ma.GetHashCode().ToString(),
				Periods.GetHashCode().ToString(),
				this.BandMode.ToString(),
				this.PeriodK.ToString(),
				src.GetHashCode().ToString(),
				stdevs.GetHashCode().ToString()
			}, () => this.GenBBands4Inp(src, Ma, Periods, stdevs, this.BandMode, this.PeriodK));
		}

		// Token: 0x060007FD RID: 2045 RVA: 0x00022334 File Offset: 0x00020534
		public IList<double> GenBBands4Inp(IList<double> price, IList<double> MA, IList<double> periods, IList<double> stdevs, int _BandMode = 1, double _PeriodK = 1.0)
		{
			if (price.Count != periods.Count)
			{
				return null;
			}
			double[] array = new double[price.Count];
			for (int i = 0; i < price.Count; i++)
			{
				array[i] = this.iBBands4In(price, MA, periods[i] * this.PeriodK, stdevs[i], _BandMode, i);
			}
			return array;
		}

		// Token: 0x060007FE RID: 2046 RVA: 0x00022394 File Offset: 0x00020594
		public double iBBands4In(IList<double> price, IList<double> _MA, double BandsPeriod, double BandsDeviation, int _BandMode, int barNum)
		{
			if ((double)barNum < BandsPeriod)
			{
				BandsPeriod = (double)barNum;
			}
			double num = _MA[barNum];
			if (_BandMode == 1)
			{
				return num + BandsDeviation;
			}
			if (_BandMode == 2)
			{
				return num - BandsDeviation;
			}
			return num;
		}

		// Token: 0x17000284 RID: 644
		[HandlerParameter(true, "1", Min = "0", Max = "2", Step = "1", Name = "Mode:\n0-ma,1-top,2-btm")]
		public int BandMode
		{
			// Token: 0x060007F9 RID: 2041 RVA: 0x0002230F File Offset: 0x0002050F
			get;
			// Token: 0x060007FA RID: 2042 RVA: 0x00022317 File Offset: 0x00020517
			set;
		}

		// Token: 0x17000286 RID: 646
		public IContext Context
		{
			// Token: 0x06000800 RID: 2048 RVA: 0x000224E9 File Offset: 0x000206E9
			get;
			// Token: 0x06000801 RID: 2049 RVA: 0x000224F1 File Offset: 0x000206F1
			set;
		}

		// Token: 0x17000285 RID: 645
		[HandlerParameter(true, "1", Min = "1", Max = "9", Step = "0.5")]
		public double PeriodK
		{
			// Token: 0x060007FB RID: 2043 RVA: 0x00022320 File Offset: 0x00020520
			get;
			// Token: 0x060007FC RID: 2044 RVA: 0x00022328 File Offset: 0x00020528
			set;
		}
	}
}
