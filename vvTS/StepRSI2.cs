using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000134 RID: 308
	[HandlerCategory("vvRSI"), HandlerName("StepRSI2")]
	public class StepRSI2 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000930 RID: 2352 RVA: 0x00026AC8 File Offset: 0x00024CC8
		public IList<double> Execute(IList<double> src)
		{
			return StepRSI2.GenStepRSI2(src, this.Context, this.RSIperiod, this.StepSize, this.Smooth, this.Chart, this.GetSignals, this.Mode1, this.StepRSIfast);
		}

		// Token: 0x0600092F RID: 2351 RVA: 0x0002669C File Offset: 0x0002489C
		public static IList<double> GenStepRSI2(IList<double> src, IContext Ctx, int _RSIperiod, int _StepSize, int _Smooth, bool _Chart, bool _GetSignals, bool _Mode1, bool _StepRSIfast)
		{
			int count = src.Count;
			int rSIperiod = _RSIperiod;
			IList<double> data = Ctx.GetData("rsi", new string[]
			{
				_RSIperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, _RSIperiod));
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			for (int i = rSIperiod; i < src.Count; i++)
			{
				double num = 0.0;
				double num2 = 0.0;
				for (int j = 0; j < _RSIperiod; j++)
				{
					double num3 = src[i - j] - src[i - j - 1];
					if (num3 > 0.0)
					{
						num2 += num3;
					}
					else
					{
						num -= num3;
					}
				}
				double num4 = num2 / (double)_RSIperiod;
				double num5 = num / (double)_RSIperiod;
				if (_Mode1)
				{
					if (num5 == 0.0)
					{
						data[i] = 100.0;
					}
					else
					{
						data[i] = 100.0 - 100.0 / (1.0 + num4 / num5);
					}
				}
				double num6 = 0.0;
				array3[i] = data[i] + 2.0 * (double)_StepSize;
				array4[i] = data[i] - 2.0 * (double)_StepSize;
				array5[i] = array5[i - 1];
				if (array5[i - 1] <= 0.0 && data[i] > array3[i - 1])
				{
					array5[i] = 1.0;
				}
				if (array5[i - 1] >= 0.0 && data[i] < array4[i - 1])
				{
					array5[i] = -1.0;
				}
				if (array5[i] > 0.0)
				{
					if (array4[i] < array4[i - 1])
					{
						array4[i] = array4[i - 1];
					}
					num6 = array4[i] + (double)_StepSize;
				}
				else if (array5[i] < 0.0)
				{
					if (array3[i] > array3[i - 1])
					{
						array3[i] = array3[i - 1];
					}
					num6 = array3[i] - (double)_StepSize;
				}
				array[i] = num6;
			}
			IList<double> list = JMA.GenJMA(data, _Smooth, 100);
			if (_GetSignals)
			{
				for (int k = 1; k < src.Count; k++)
				{
					array2[0] = 0.0;
					if (list[k] > array[k] && list[k - 1] < array[k - 1])
					{
						array2[k] = 1.0;
					}
					if (list[k] < array[k] && list[k - 1] > array[k - 1])
					{
						array2[k] = -1.0;
					}
				}
			}
			if (_Chart)
			{
				IPane pane = Ctx.CreatePane("StepRSI", 25.0, false, false);
				pane.AddList("", list, 0, 13371596, 0, 0);
				IGraphList graphList = pane.AddList(string.Concat(new string[]
				{
					"StepRSI(",
					_RSIperiod.ToString(),
					",",
					_StepSize.ToString(),
					",",
					_Smooth.ToString(),
					")"
				}), array, 0, 264127, 0, 0);
				graphList.set_Thickness(2);
			}
			if (_GetSignals)
			{
				return array2;
			}
			if (!_StepRSIfast)
			{
				return array;
			}
			return list;
		}

		// Token: 0x170002F4 RID: 756
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Chart
		{
			// Token: 0x06000927 RID: 2343 RVA: 0x0002663D File Offset: 0x0002483D
			get;
			// Token: 0x06000928 RID: 2344 RVA: 0x00026645 File Offset: 0x00024845
			set;
		}

		// Token: 0x170002F8 RID: 760
		public IContext Context
		{
			// Token: 0x06000931 RID: 2353 RVA: 0x00026B0B File Offset: 0x00024D0B
			get;
			// Token: 0x06000932 RID: 2354 RVA: 0x00026B13 File Offset: 0x00024D13
			set;
		}

		// Token: 0x170002F5 RID: 757
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool GetSignals
		{
			// Token: 0x06000929 RID: 2345 RVA: 0x0002664E File Offset: 0x0002484E
			get;
			// Token: 0x0600092A RID: 2346 RVA: 0x00026656 File Offset: 0x00024856
			set;
		}

		// Token: 0x170002F6 RID: 758
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Mode1
		{
			// Token: 0x0600092B RID: 2347 RVA: 0x0002665F File Offset: 0x0002485F
			get;
			// Token: 0x0600092C RID: 2348 RVA: 0x00026667 File Offset: 0x00024867
			set;
		}

		// Token: 0x170002F1 RID: 753
		[HandlerParameter(true, "14", Min = "5", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x06000921 RID: 2337 RVA: 0x0002660A File Offset: 0x0002480A
			get;
			// Token: 0x06000922 RID: 2338 RVA: 0x00026612 File Offset: 0x00024812
			set;
		}

		// Token: 0x170002F3 RID: 755
		[HandlerParameter(true, "1", Min = "1", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000925 RID: 2341 RVA: 0x0002662C File Offset: 0x0002482C
			get;
			// Token: 0x06000926 RID: 2342 RVA: 0x00026634 File Offset: 0x00024834
			set;
		}

		// Token: 0x170002F7 RID: 759
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool StepRSIfast
		{
			// Token: 0x0600092D RID: 2349 RVA: 0x00026670 File Offset: 0x00024870
			get;
			// Token: 0x0600092E RID: 2350 RVA: 0x00026678 File Offset: 0x00024878
			set;
		}

		// Token: 0x170002F2 RID: 754
		[HandlerParameter(true, "5", Min = "1", Max = "20", Step = "1")]
		public int StepSize
		{
			// Token: 0x06000923 RID: 2339 RVA: 0x0002661B File Offset: 0x0002481B
			get;
			// Token: 0x06000924 RID: 2340 RVA: 0x00026623 File Offset: 0x00024823
			set;
		}
	}
}
