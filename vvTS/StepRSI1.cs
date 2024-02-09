using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000133 RID: 307
	[HandlerCategory("vvRSI"), HandlerName("Step RSI")]
	public class StepRSI1 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600091D RID: 2333 RVA: 0x000265E8 File Offset: 0x000247E8
		public IList<double> Execute(IList<double> src)
		{
			return this.GenStepRSI(src);
		}

		// Token: 0x0600091C RID: 2332 RVA: 0x000262C4 File Offset: 0x000244C4
		public IList<double> GenStepRSI(IList<double> src)
		{
			int num = 0;
			int num2 = 0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			double num6 = 0.0;
			IList<double> data = this.Context.GetData("rsi", new string[]
			{
				this.RSIperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, this.RSIperiod));
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			int arg_E3_0 = src.Count;
			for (int i = 0; i < src.Count; i++)
			{
				double num7 = data[i] + (double)(2 * this.StepSizeFast);
				double num8 = data[i] - (double)(2 * this.StepSizeFast);
				if (data[i] > num4)
				{
					num = 1;
				}
				if (data[i] < num3)
				{
					num = -1;
				}
				if (num > 0 && num8 < num3)
				{
					num8 = num3;
				}
				if (num < 0 && num7 > num4)
				{
					num7 = num4;
				}
				double num9 = data[i] + (double)(2 * this.StepSizeSlow);
				double num10 = data[i] - (double)(2 * this.StepSizeSlow);
				if (data[i] > num6)
				{
					num2 = 1;
				}
				if (data[i] < num5)
				{
					num2 = -1;
				}
				if (num2 > 0 && num10 < num5)
				{
					num10 = num5;
				}
				if (num2 < 0 && num9 > num6)
				{
					num9 = num6;
				}
				if (num > 0)
				{
					array[i] = num8 + (double)this.StepSizeFast;
				}
				if (num < 0)
				{
					array[i] = num7 - (double)this.StepSizeFast;
				}
				if (num2 > 0)
				{
					array2[i] = num10 + (double)this.StepSizeSlow;
				}
				if (num2 < 0)
				{
					array2[i] = num9 - (double)this.StepSizeSlow;
				}
				num3 = num8;
				num4 = num7;
				num5 = num10;
				num6 = num9;
			}
			IList<double> list = JMA.GenJMA(data, this.Smooth, 100);
			if (this.Chart)
			{
				IPane pane = this.Context.CreatePane("StepRSI", 30.0, false, false);
				pane.AddList("RSI(" + this.RSIperiod.ToString() + ")", list, 0, 14554121, 3, 0);
				pane.AddList("StRSIfast(" + this.StepSizeFast.ToString() + ")", array, 0, 329128, 3, 0);
				pane.AddList("StRSIslow(" + this.StepSizeSlow.ToString() + ")", array2, 0, 329128, 0, 0);
			}
			if (!this.StepRSIfast && !this.StepRSIslow)
			{
				return list;
			}
			if (!this.StepRSIfast)
			{
				return array2;
			}
			return array;
		}

		// Token: 0x170002ED RID: 749
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Chart
		{
			// Token: 0x06000916 RID: 2326 RVA: 0x00026271 File Offset: 0x00024471
			get;
			// Token: 0x06000917 RID: 2327 RVA: 0x00026279 File Offset: 0x00024479
			set;
		}

		// Token: 0x170002F0 RID: 752
		public IContext Context
		{
			// Token: 0x0600091E RID: 2334 RVA: 0x000265F1 File Offset: 0x000247F1
			get;
			// Token: 0x0600091F RID: 2335 RVA: 0x000265F9 File Offset: 0x000247F9
			set;
		}

		// Token: 0x170002E9 RID: 745
		[HandlerParameter(true, "14", Min = "5", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x0600090E RID: 2318 RVA: 0x0002622D File Offset: 0x0002442D
			get;
			// Token: 0x0600090F RID: 2319 RVA: 0x00026235 File Offset: 0x00024435
			set;
		}

		// Token: 0x170002EC RID: 748
		[HandlerParameter(true, "3", Min = "1", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000914 RID: 2324 RVA: 0x00026260 File Offset: 0x00024460
			get;
			// Token: 0x06000915 RID: 2325 RVA: 0x00026268 File Offset: 0x00024468
			set;
		}

		// Token: 0x170002EE RID: 750
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool StepRSIfast
		{
			// Token: 0x06000918 RID: 2328 RVA: 0x00026282 File Offset: 0x00024482
			get;
			// Token: 0x06000919 RID: 2329 RVA: 0x0002628A File Offset: 0x0002448A
			set;
		}

		// Token: 0x170002EF RID: 751
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool StepRSIslow
		{
			// Token: 0x0600091A RID: 2330 RVA: 0x00026293 File Offset: 0x00024493
			get;
			// Token: 0x0600091B RID: 2331 RVA: 0x0002629B File Offset: 0x0002449B
			set;
		}

		// Token: 0x170002EA RID: 746
		[HandlerParameter(true, "5", Min = "5", Max = "20", Step = "1")]
		public int StepSizeFast
		{
			// Token: 0x06000910 RID: 2320 RVA: 0x0002623E File Offset: 0x0002443E
			get;
			// Token: 0x06000911 RID: 2321 RVA: 0x00026246 File Offset: 0x00024446
			set;
		}

		// Token: 0x170002EB RID: 747
		[HandlerParameter(true, "15", Min = "10", Max = "30", Step = "1")]
		public int StepSizeSlow
		{
			// Token: 0x06000912 RID: 2322 RVA: 0x0002624F File Offset: 0x0002444F
			get;
			// Token: 0x06000913 RID: 2323 RVA: 0x00026257 File Offset: 0x00024457
			set;
		}
	}
}
