using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200011C RID: 284
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("BBand HL")]
	public class BBandsHL : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600080E RID: 2062 RVA: 0x0002279C File Offset: 0x0002099C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("bbHLvolty", new string[]
			{
				this.BaseMaPeriod.ToString(),
				this.HLvoltyMAperiod.ToString(),
				this.Factor.ToString(),
				this.BandMode.ToString(),
				this.MaMode.ToString(),
				sec.get_CacheName()
			}, () => BBandsHL.GenBBandsHL(sec, this.Context, this.BaseMaPeriod, this.HLvoltyMAperiod, this.Factor, this.BandMode, this.MaMode));
		}

		// Token: 0x0600080D RID: 2061 RVA: 0x00022558 File Offset: 0x00020758
		public static IList<double> GenBBandsHL(ISecurity src, IContext ctx, int _BaseMaPeriod, int _HLvoltyMAperiod, double _Factor, int _BandMode, int _MaMode)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			IList<double> list4 = new double[count];
			list4[0] = (list3[0] = highPrices[0] - lowPrices[0]);
			list2[0] = (highPrices[0] + lowPrices[0]) / 2.0;
			for (int i = 1; i < count; i++)
			{
				list3[i] = highPrices[i] - lowPrices[i];
				list4[i] = ((_MaMode == 1) ? EMA.iEMA(list3, list4, (i < _HLvoltyMAperiod) ? i : _HLvoltyMAperiod, i) : SMA.iSMA(list3, (i < _HLvoltyMAperiod) ? i : _HLvoltyMAperiod, i));
				list2[i] = ((_MaMode == 1) ? EMA.iEMA(closePrices, list2, (i < _BaseMaPeriod) ? i : _BaseMaPeriod, i) : SMA.iSMA(closePrices, (i < _BaseMaPeriod) ? i : _BaseMaPeriod, i));
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

		// Token: 0x1700028A RID: 650
		[HandlerParameter(false, "1", NotOptimized = true, Name = "BandMode:\n1-top,2-bottom,0-basema")]
		public int BandMode
		{
			// Token: 0x06000809 RID: 2057 RVA: 0x00022535 File Offset: 0x00020735
			get;
			// Token: 0x0600080A RID: 2058 RVA: 0x0002253D File Offset: 0x0002073D
			set;
		}

		// Token: 0x17000287 RID: 647
		[HandlerParameter(true, "10", Min = "10", Max = "20", Step = "1")]
		public int BaseMaPeriod
		{
			// Token: 0x06000803 RID: 2051 RVA: 0x00022502 File Offset: 0x00020702
			get;
			// Token: 0x06000804 RID: 2052 RVA: 0x0002250A File Offset: 0x0002070A
			set;
		}

		// Token: 0x1700028C RID: 652
		public IContext Context
		{
			// Token: 0x0600080F RID: 2063 RVA: 0x00022847 File Offset: 0x00020A47
			get;
			// Token: 0x06000810 RID: 2064 RVA: 0x0002284F File Offset: 0x00020A4F
			set;
		}

		// Token: 0x17000289 RID: 649
		[HandlerParameter(true, "2", Min = "1", Max = "9", Step = "0.1")]
		public double Factor
		{
			// Token: 0x06000807 RID: 2055 RVA: 0x00022524 File Offset: 0x00020724
			get;
			// Token: 0x06000808 RID: 2056 RVA: 0x0002252C File Offset: 0x0002072C
			set;
		}

		// Token: 0x17000288 RID: 648
		[HandlerParameter(true, "30", Min = "10", Max = "40", Step = "1")]
		public int HLvoltyMAperiod
		{
			// Token: 0x06000805 RID: 2053 RVA: 0x00022513 File Offset: 0x00020713
			get;
			// Token: 0x06000806 RID: 2054 RVA: 0x0002251B File Offset: 0x0002071B
			set;
		}

		// Token: 0x1700028B RID: 651
		[HandlerParameter(true, "1", Min = "0", Max = "8", Step = "1", Name = "MaMode:\n0-sma,1-ema")]
		public int MaMode
		{
			// Token: 0x0600080B RID: 2059 RVA: 0x00022546 File Offset: 0x00020746
			get;
			// Token: 0x0600080C RID: 2060 RVA: 0x0002254E File Offset: 0x0002074E
			set;
		}
	}
}
