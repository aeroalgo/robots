using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200011D RID: 285
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("BBand HL - 3in HLC"), InputsCount(3)]
	public class BBandsHL3HLC : IThreeSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600081D RID: 2077 RVA: 0x00022AE8 File Offset: 0x00020CE8
		public IList<double> Execute(IList<double> High, IList<double> Low, IList<double> Close)
		{
			return this.Context.GetData("bbHLvolty3in", new string[]
			{
				High.GetHashCode().ToString(),
				Low.GetHashCode().ToString(),
				Close.GetHashCode().ToString(),
				this.BaseMaPeriod.ToString(),
				this.HLvoltyMAperiod.ToString(),
				this.Factor.ToString(),
				this.BandMode.ToString(),
				this.MaMode.ToString()
			}, () => BBandsHL3HLC.GenBBandsHL(High, Low, Close, this.Context, this.BaseMaPeriod, this.HLvoltyMAperiod, this.Factor, this.BandMode, this.MaMode));
		}

		// Token: 0x0600081C RID: 2076 RVA: 0x000228B8 File Offset: 0x00020AB8
		public static IList<double> GenBBandsHL(IList<double> High, IList<double> Low, IList<double> Close, IContext ctx, int _BaseMaPeriod, int _HLvoltyMAperiod, double _Factor, int _BandMode, int _MaMode)
		{
			int count = Close.Count;
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			IList<double> list4 = new double[count];
			list4[0] = (list3[0] = High[0] - Low[0]);
			list2[0] = (High[0] + Low[0]) / 2.0;
			for (int i = 1; i < count; i++)
			{
				list3[i] = High[i] - Low[i];
				list4[i] = ((_MaMode == 1) ? EMA.iEMA(list3, list4, (i < _HLvoltyMAperiod) ? i : _HLvoltyMAperiod, i) : SMA.iSMA(list3, (i < _HLvoltyMAperiod) ? i : _HLvoltyMAperiod, i));
				list2[i] = ((_MaMode == 1) ? EMA.iEMA(Close, list2, (i < _BaseMaPeriod) ? i : _BaseMaPeriod, i) : SMA.iSMA(Close, (i < _BaseMaPeriod) ? i : _BaseMaPeriod, i));
				double num = 0.0;
				int j = i - _HLvoltyMAperiod + 1;
				if (j < 0)
				{
					j = 0;
				}
				double num2 = list2[i];
				while (j <= i)
				{
					double num3 = list4[j] - num2;
					num += num3 * num3;
					j++;
				}
				double num4 = _Factor * Math.Sqrt(num / (double)_HLvoltyMAperiod);
				list[i] = list2[i];
				if (_BandMode == 1)
				{
					IList<double> list5;
					int index;
					(list5 = list)[index = i] = list5[index] + num4;
				}
				if (_BandMode == 2)
				{
					IList<double> list6;
					int index2;
					(list6 = list)[index2 = i] = list6[index2] - num4;
				}
			}
			return list;
		}

		// Token: 0x17000290 RID: 656
		[HandlerParameter(false, "1", NotOptimized = true, Name = "BandMode:\n1-top,2-bottom,0-basema")]
		public int BandMode
		{
			// Token: 0x06000818 RID: 2072 RVA: 0x00022893 File Offset: 0x00020A93
			get;
			// Token: 0x06000819 RID: 2073 RVA: 0x0002289B File Offset: 0x00020A9B
			set;
		}

		// Token: 0x1700028D RID: 653
		[HandlerParameter(true, "10", Min = "10", Max = "20", Step = "1")]
		public int BaseMaPeriod
		{
			// Token: 0x06000812 RID: 2066 RVA: 0x00022860 File Offset: 0x00020A60
			get;
			// Token: 0x06000813 RID: 2067 RVA: 0x00022868 File Offset: 0x00020A68
			set;
		}

		// Token: 0x17000292 RID: 658
		public IContext Context
		{
			// Token: 0x0600081E RID: 2078 RVA: 0x00022BD8 File Offset: 0x00020DD8
			get;
			// Token: 0x0600081F RID: 2079 RVA: 0x00022BE0 File Offset: 0x00020DE0
			set;
		}

		// Token: 0x1700028F RID: 655
		[HandlerParameter(true, "0.002", Min = "0.001", Max = "0.01", Step = "0.001")]
		public double Factor
		{
			// Token: 0x06000816 RID: 2070 RVA: 0x00022882 File Offset: 0x00020A82
			get;
			// Token: 0x06000817 RID: 2071 RVA: 0x0002288A File Offset: 0x00020A8A
			set;
		}

		// Token: 0x1700028E RID: 654
		[HandlerParameter(true, "30", Min = "10", Max = "40", Step = "1")]
		public int HLvoltyMAperiod
		{
			// Token: 0x06000814 RID: 2068 RVA: 0x00022871 File Offset: 0x00020A71
			get;
			// Token: 0x06000815 RID: 2069 RVA: 0x00022879 File Offset: 0x00020A79
			set;
		}

		// Token: 0x17000291 RID: 657
		[HandlerParameter(true, "1", Min = "0", Max = "8", Step = "1", Name = "MaMode:\n0-sma,1-ema")]
		public int MaMode
		{
			// Token: 0x0600081A RID: 2074 RVA: 0x000228A4 File Offset: 0x00020AA4
			get;
			// Token: 0x0600081B RID: 2075 RVA: 0x000228AC File Offset: 0x00020AAC
			set;
		}
	}
}
