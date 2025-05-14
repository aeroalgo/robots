using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200014A RID: 330
	[HandlerCategory("vvMACD"), HandlerName("MACDHistoAdv")]
	public class MACDHistoAdv : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A2B RID: 2603 RVA: 0x0002A468 File Offset: 0x00028668
		public IList<double> Execute(IList<double> _src)
		{
			IList<double> fastEMA = this.Context.GetData("ema", new string[]
			{
				this.EMAfp.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.EMA(_src, this.EMAfp));
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				this.EMAfp.ToString(),
				fastEMA.GetHashCode().ToString()
			}, () => Series.EMA(fastEMA, this.EMAfp));
			IList<double> slowEMA = this.Context.GetData("ema", new string[]
			{
				this.EMAsp.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.EMA(_src, this.EMAsp));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				this.EMAsp.ToString(),
				slowEMA.GetHashCode().ToString()
			}, () => Series.EMA(slowEMA, this.EMAsp));
			IList<double> list = new List<double>(_src.Count);
			IList<double> list2 = new List<double>(_src.Count);
			IList<double> list3 = new List<double>(_src.Count);
			IList<double> list4 = new List<double>(_src.Count);
			IList<double> list5 = new List<double>(_src.Count);
			for (int i = 0; i < _src.Count; i++)
			{
				double num = 2.0 * fastEMA[i];
				double num2 = 2.0 * slowEMA[i];
				double num3 = num - num2;
				double num4 = num3 - data[i];
				double item = num4 + data2[i];
				list.Add(item);
			}
			IList<double> list6 = Series.EMA(list, this.EMAsignal);
			IList<double> list7 = Series.EMA(list6, this.EMAsignal);
			for (int j = 0; j < _src.Count; j++)
			{
				double num3 = 2.0 * list6[j];
				double num4 = list7[j];
				list2.Add(num3 - num4);
			}
			for (int k = 0; k < _src.Count; k++)
			{
				double num3 = list[k];
				double num4 = list2[k];
				double item = num3 - num4;
				list3.Add(item);
			}
			IList<double> list8 = Series.EMA(list3, this.EMANoiseFilter);
			for (int l = 0; l < _src.Count; l++)
			{
				if (list8[l] > list3[l])
				{
					list5.Add(list3[l]);
					list4.Add(0.0);
				}
				else
				{
					list5.Add(0.0);
					list4.Add(list3[l]);
				}
			}
			if (this.Chart)
			{
				IPane pane = this.Context.CreatePane("MACDHistoAdv", 25.0, false, false);
				pane.AddList(string.Format("MACD({0},{1},{2})", this.EMAfp, this.EMAsp, this.EMAsignal), list5, 3, 14228740, 0, 0);
				pane.AddList(string.Format(".", new object[0]), list4, 3, 307983, 0, 0);
			}
			if (!this.RedHistogram)
			{
				return list4;
			}
			return list5;
		}

		// Token: 0x17000355 RID: 853
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Chart
		{
			// Token: 0x06000A29 RID: 2601 RVA: 0x0002A3EC File Offset: 0x000285EC
			get;
			// Token: 0x06000A2A RID: 2602 RVA: 0x0002A3F4 File Offset: 0x000285F4
			set;
		}

		// Token: 0x17000356 RID: 854
		public IContext Context
		{
			// Token: 0x06000A2C RID: 2604 RVA: 0x0002A87D File Offset: 0x00028A7D
			get;
			// Token: 0x06000A2D RID: 2605 RVA: 0x0002A885 File Offset: 0x00028A85
			set;
		}

		// Token: 0x17000350 RID: 848
		[HandlerParameter(true, "12", Min = "5", Max = "120", Step = "1")]
		public int EMAfp
		{
			// Token: 0x06000A1F RID: 2591 RVA: 0x0002A397 File Offset: 0x00028597
			get;
			// Token: 0x06000A20 RID: 2592 RVA: 0x0002A39F File Offset: 0x0002859F
			set;
		}

		// Token: 0x17000353 RID: 851
		[HandlerParameter(true, "5", Min = "5", Max = "100", Step = "1")]
		public int EMANoiseFilter
		{
			// Token: 0x06000A25 RID: 2597 RVA: 0x0002A3CA File Offset: 0x000285CA
			get;
			// Token: 0x06000A26 RID: 2598 RVA: 0x0002A3D2 File Offset: 0x000285D2
			set;
		}

		// Token: 0x17000352 RID: 850
		[HandlerParameter(true, "9", Min = "5", Max = "100", Step = "1")]
		public int EMAsignal
		{
			// Token: 0x06000A23 RID: 2595 RVA: 0x0002A3B9 File Offset: 0x000285B9
			get;
			// Token: 0x06000A24 RID: 2596 RVA: 0x0002A3C1 File Offset: 0x000285C1
			set;
		}

		// Token: 0x17000351 RID: 849
		[HandlerParameter(true, "26", Min = "20", Max = "200", Step = "1")]
		public int EMAsp
		{
			// Token: 0x06000A21 RID: 2593 RVA: 0x0002A3A8 File Offset: 0x000285A8
			get;
			// Token: 0x06000A22 RID: 2594 RVA: 0x0002A3B0 File Offset: 0x000285B0
			set;
		}

		// Token: 0x17000354 RID: 852
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool RedHistogram
		{
			// Token: 0x06000A27 RID: 2599 RVA: 0x0002A3DB File Offset: 0x000285DB
			get;
			// Token: 0x06000A28 RID: 2600 RVA: 0x0002A3E3 File Offset: 0x000285E3
			set;
		}
	}
}
