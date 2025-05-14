using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200011A RID: 282
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("BBands 3inp"), InputInfo(0, "Цена"), InputInfo(2, "Период"), InputInfo(1, "Средняя"), InputsCount(3)]
	public class BBands3In : IThreeSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060007F5 RID: 2037 RVA: 0x0002223C File Offset: 0x0002043C
		public IList<double> Execute(IList<double> src, IList<double> Ma, IList<double> Periods)
		{
			return this.Context.GetData("bb3in", new string[]
			{
				Ma.ToString(),
				Periods.ToString(),
				this.BandsDeviation.ToString(),
				this.BandMode.ToString(),
				this.PeriodK.ToString(),
				src.GetHashCode().ToString()
			}, () => this.GenBBands3Inp(src, Ma, Periods, this.BandsDeviation, this.BandMode, this.PeriodK));
		}

		// Token: 0x060007F3 RID: 2035 RVA: 0x00022128 File Offset: 0x00020328
		public IList<double> GenBBands3Inp(IList<double> price, IList<double> MA, IList<double> periods, double _BandsDeviation, int _BandMode, double _PeriodK)
		{
			if (price.Count != periods.Count)
			{
				return null;
			}
			double[] array = new double[price.Count];
			for (int i = 0; i < price.Count; i++)
			{
				array[i] = this.iBBands3In(price, MA, periods[i] * this.PeriodK, _BandsDeviation, _BandMode, i);
			}
			return array;
		}

		// Token: 0x060007F4 RID: 2036 RVA: 0x00022184 File Offset: 0x00020384
		public double iBBands3In(IList<double> price, IList<double> _MA, double BandsPeriod, double BandsDeviation, int _BandMode, int barNum)
		{
			if ((double)barNum < BandsPeriod)
			{
				BandsPeriod = (double)barNum;
			}
			double num = 0.0;
			int i = barNum - (int)BandsPeriod + 1;
			double num2 = _MA[barNum];
			while (i <= barNum)
			{
				double num3 = price[i] - num2;
				num += num3 * num3;
				i++;
			}
			double num4 = BandsDeviation * Math.Sqrt(num / BandsPeriod);
			if (_BandMode == 1)
			{
				return num2 + num4;
			}
			if (_BandMode == 2)
			{
				return num2 - num4;
			}
			return num2;
		}

		// Token: 0x17000281 RID: 641
		[HandlerParameter(true, "1", Min = "0", Max = "2", Step = "1", Name = "Mode:\n0-ma,1-top,2-btm")]
		public int BandMode
		{
			// Token: 0x060007EF RID: 2031 RVA: 0x00022106 File Offset: 0x00020306
			get;
			// Token: 0x060007F0 RID: 2032 RVA: 0x0002210E File Offset: 0x0002030E
			set;
		}

		// Token: 0x17000280 RID: 640
		[HandlerParameter(true, "2", Min = "0.5", Max = "3", Step = "0.1")]
		public double BandsDeviation
		{
			// Token: 0x060007ED RID: 2029 RVA: 0x000220F5 File Offset: 0x000202F5
			get;
			// Token: 0x060007EE RID: 2030 RVA: 0x000220FD File Offset: 0x000202FD
			set;
		}

		// Token: 0x17000283 RID: 643
		public IContext Context
		{
			// Token: 0x060007F6 RID: 2038 RVA: 0x000222F6 File Offset: 0x000204F6
			get;
			// Token: 0x060007F7 RID: 2039 RVA: 0x000222FE File Offset: 0x000204FE
			set;
		}

		// Token: 0x17000282 RID: 642
		[HandlerParameter(true, "1", Min = "1", Max = "9", Step = "0.5")]
		public double PeriodK
		{
			// Token: 0x060007F1 RID: 2033 RVA: 0x00022117 File Offset: 0x00020317
			get;
			// Token: 0x060007F2 RID: 2034 RVA: 0x0002211F File Offset: 0x0002031F
			set;
		}
	}
}
